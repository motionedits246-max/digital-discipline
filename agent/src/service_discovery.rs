//! Small, bounded classification ladder: remembered/manual -> local metadata -> optional public lookup -> owner-approved AI.
use axum::{extract::State,http::StatusCode,Json};use rusqlite::{Connection,params,OptionalExtension};use serde::Deserialize;use serde_json::{json,Value};use std::{sync::{Arc,atomic::Ordering},net::IpAddr,time::Duration};
use crate::{AppState,ApiResult,features,classification_store,classification,data_lifecycle,err,now_ms};
pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS service_discovery(subject TEXT PRIMARY KEY,category TEXT NOT NULL DEFAULT 'Unclassified',source TEXT NOT NULL DEFAULT 'pending',description TEXT NOT NULL DEFAULT '',reason TEXT NOT NULL DEFAULT '',first_ms INTEGER NOT NULL,last_ms INTEGER NOT NULL,checked INTEGER NOT NULL DEFAULT 0,prompt_seen_ms INTEGER NOT NULL DEFAULT 0,llm_approved INTEGER NOT NULL DEFAULT 0,llm_revision TEXT NOT NULL DEFAULT '');
 CREATE TABLE IF NOT EXISTS service_lookup_requests(at_ms INTEGER NOT NULL,subject TEXT NOT NULL);CREATE INDEX IF NOT EXISTS idx_service_lookup_time ON service_lookup_requests(at_ms);
 INSERT OR IGNORE INTO settings VALUES('service_public_lookup','false');INSERT OR IGNORE INTO settings VALUES('service_overlay_enabled','true');")?;
 db.execute("INSERT OR IGNORE INTO settings VALUES('service_discovery_since',?1)",params![now_ms().to_string()])?;
 // Accepted decisions already known before this feature do not become new prompts.
 db.execute("INSERT OR IGNORE INTO service_discovery(subject,category,source,reason,first_ms,last_ms,checked,prompt_seen_ms) SELECT subject,category,'remembered',reason,updated_ms,last_used_ms,1,last_used_ms FROM classification_memory WHERE state='accepted'",[])?;Ok(())
}
fn excluded(db:&Connection,subject:&str)->bool{if let Some(a)=subject.strip_prefix("app:"){features::excluded(db,a,None)}else{features::excluded(db,"",subject.strip_prefix("domain:"))}}
pub fn discover(db:&Connection,at:i64)->rusqlite::Result<()>{
 let since=features::read_setting(db,"service_discovery_since").parse::<i64>().unwrap_or(at);
 let mut q=db.prepare("SELECT 'app:'||lower(app),MAX(end_ms) FROM sessions WHERE end_ms>=?1 GROUP BY app UNION ALL SELECT 'domain:'||domain,MAX(end_ms) FROM sessions WHERE domain IS NOT NULL AND end_ms>=?1 GROUP BY domain ORDER BY 2 DESC LIMIT 300")?;
 let rows=q.query_map(params![since.max(at-30000)],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
 for (subject,last) in rows{if !classification::safe_subject(&subject)||excluded(db,&subject)||subject.contains("digital-discipline")||["app:lockapp.exe","app:logonui.exe","app:dwm.exe"].contains(&subject.as_str()){continue;}
  db.execute("INSERT INTO service_discovery(subject,first_ms,last_ms) VALUES(?1,?2,?2) ON CONFLICT(subject) DO UPDATE SET last_ms=max(last_ms,excluded.last_ms)",params![subject,last])?;
 }Ok(())
}
pub fn clean_description(text:&str)->String{let text=text.split_whitespace().collect::<Vec<_>>().join(" ");if text.contains(":\\")||text.contains("\\\\")||text.contains('@'){return String::new();}text.chars().filter(|c|!c.is_control()).take(240).collect()}
pub fn infer(description:&str)->Option<&'static str>{
 let text=format!(" {} ",description.to_ascii_lowercase().chars().map(|c|if c.is_ascii_alphanumeric(){c}else{' '}).collect::<String>().split_whitespace().collect::<Vec<_>>().join(" "));
 let rules:[(&str,&[(&str,u32)]);10]=[
  ("Development",&[("development environment",9),("code editor",8),("software development",8),("programming",5),("development",5),(" ide ",5),("developer",4)]),
  ("Design",&[("video editor",9),("graphic design",9),("photo editor",9),("animation",5),("illustration",5)]),
  ("Education",&[("online courses",8),("learning platform",8),("education",5),("tutorials",4)]),
  ("Productivity",&[("project management",8),("task management",8),("spreadsheet",6),("word processor",6),("note taking",6)]),
  ("Communication",&[("video conferencing",8),("team messaging",8),("email client",8),("messaging app",7)]),
  ("Entertainment",&[("video streaming",8),("music streaming",8),("media player",7),("entertainment",5)]),
  ("Gaming",&[("video game",8),("game launcher",8),("gaming platform",8)]),
  ("Social",&[("social network",8),("social media",7)]),("News",&[("news publisher",8),("news website",8),("journalism",5)]),
  ("Browser",&[("web browser",9),("internet browser",9)])];
 let mut scores=rules.iter().map(|(category,keys)|(*category,keys.iter().filter(|(word,_)|text.contains(&format!(" {} ",word.trim()))).map(|(_,w)|*w).max().unwrap_or(0))).collect::<Vec<_>>();scores.sort_by_key(|(_,s)|std::cmp::Reverse(*s));
 if scores[0].1>=5&&scores[0].1.saturating_sub(scores[1].1)>=3{Some(scores[0].0)}else{None}
}
pub fn apply_evidence(db:&Connection,subject:&str,description:&str,source:&str,at:i64)->rusqlite::Result<()>{
 if excluded(db,subject){return Ok(());}
 let description=clean_description(description);
 let manual:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM category_overrides WHERE lower(subject)=?1)",params![subject],|r|r.get(0))?;
 let prior=if let Some(app)=subject.strip_prefix("app:"){features::category_for(db,app,None)}else{features::category_for(db,"",subject.strip_prefix("domain:"))};
 let category=if prior!="Unclassified"{prior.clone()}else{infer(&description).unwrap_or("Unclassified").to_string()};
 let source=if manual{"your_choice"}else if prior!="Unclassified"{"remembered"}else{source};
 let reason=if category=="Unclassified"{"No reliable lightweight match; AI requires your approval"}else if manual{"Your saved category has priority"}else{"Category from remembered rules or descriptive keywords; you can change it"};
 db.execute("UPDATE service_discovery SET category=?2,source=?3,description=?4,reason=?5,checked=1 WHERE subject=?1",params![subject,category,source,description,reason])?;
 if category!="Unclassified"&&!manual&&!(source=="duckduckgo_summary"&&subject.starts_with("app:")){
  db.execute("INSERT OR IGNORE INTO classifications(subject,state,category,reason,created_ms,updated_ms) VALUES(?1,'succeeded',?2,?3,?4,?4)",params![subject,category,reason,at])?;
  classification_store::remember(db,subject,&category,0.88,reason,source,"keywords-v1",at,false)?;
 }Ok(())
}
fn public_ip(ip:IpAddr)->bool{match ip{IpAddr::V4(v)=>!v.is_private()&&!v.is_loopback()&&!v.is_link_local()&&!v.is_broadcast()&&!v.is_documentation()&&!v.is_multicast()&&!v.is_unspecified()&&v.octets()[0]!=0&&v.octets()[0]<224&&v.octets()[0]!=127&&!(v.octets()[0]==198&&(18..=19).contains(&v.octets()[1]))&&!(v.octets()[0]==192&&v.octets()[1]==0&&v.octets()[2]==0)&&!(v.octets()[0]==100&&(64..=127).contains(&v.octets()[1])),IpAddr::V6(v)=>{let s=v.segments();s[0]&0xe000==0x2000&&s[0]!=0x2002&&!(s[0]==0x2001&&s[1]==0)&&!v.is_loopback()&&!v.is_unspecified()&&s[0]&0xfe00!=0xfc00&&s[0]&0xffc0!=0xfe80&&s[0]&0xff00!=0xff00&&v.to_ipv4_mapped().is_none()&&!(s[0]==0x2001&&s[1]==0x0db8)}}}
async fn bounded_get(url:reqwest::Url,cap:usize)->Option<Vec<u8>>{
 let host=url.host_str()?.to_string();let addrs=tokio::time::timeout(Duration::from_secs(2),tokio::net::lookup_host((host.as_str(),443))).await.ok()?.ok()?.collect::<Vec<_>>();
 if addrs.is_empty()||addrs.iter().any(|a|!public_ip(a.ip())){return None;}
 let client=reqwest::Client::builder().timeout(Duration::from_secs(4)).redirect(reqwest::redirect::Policy::none()).no_proxy().resolve_to_addrs(&host,&addrs).user_agent("DigitalDiscipline/0.4 public-service-metadata").build().ok()?;
 let mut response=client.get(url).send().await.ok()?;if !response.status().is_success(){return None;}let mut data=Vec::new();while let Some(chunk)=response.chunk().await.ok()?{let remaining=cap.saturating_sub(data.len());if chunk.len()>remaining{data.extend_from_slice(&chunk[..remaining]);break;}data.extend_from_slice(&chunk);}Some(data)
}
pub fn html_description(html:&str)->String{
 let mut cursor=0;for _ in 0..80{let lower=html[cursor..].to_ascii_lowercase();let Some(start)=lower.find("<meta")else{break;};cursor+=start;let Some(end)=html[cursor..].find('>')else{break;};let tag=&html[cursor..cursor+end];let lower=tag.to_ascii_lowercase();cursor+=end+1;
  if !lower.contains("description"){continue;}
  if let Some(i)=lower.find("content="){let rest=tag[i+8..].trim_start();let Some(quote)=rest.chars().next()else{continue;};if !['\"','\''].contains(&quote){continue;}if let Some(end)=rest[1..].find(quote){return clean_description(&rest[1..1+end].replace("&amp;","&").replace("&quot;","\"").replace("&#39;","'"));}}
 }String::new()
}
fn reserve_lookup(db:&Connection,subject:&str,at:i64)->rusqlite::Result<bool>{let start=at.div_euclid(86400000)*86400000;let count:i64=db.query_row("SELECT COUNT(*) FROM service_lookup_requests WHERE at_ms>=?1",params![start],|r|r.get(0))?;if count>=20{return Ok(false);}db.execute("INSERT INTO service_lookup_requests VALUES(?1,?2)",params![at,subject])?;Ok(true)}
pub async fn tick(s:Arc<AppState>){
 let work=(||->rusqlite::Result<Option<(String,String,bool)>>{let db=s.db.lock().unwrap();if s.paused.load(Ordering::SeqCst)||data_lifecycle::is_private(&db,now_ms()){return Ok(None);}discover(&db,now_ms())?;
  let subject=db.query_row("SELECT subject FROM service_discovery WHERE checked=0 ORDER BY last_ms DESC LIMIT 1",[],|r|r.get::<_,String>(0)).optional()?;
  Ok(subject.map(|subject|(subject,features::read_setting(&db,"activity_data_epoch"),features::read_setting(&db,"service_public_lookup")=="true")))})();
 let Ok(Some((subject,epoch,lookup)))=work else{return;};
 let mut description=if let Some(app)=subject.strip_prefix("app:"){let app=app.to_string();tokio::time::timeout(Duration::from_secs(2),tokio::task::spawn_blocking(move||crate::windows::application_description(&app))).await.ok().and_then(Result::ok).unwrap_or_default()}else{String::new()};let mut source=if description.is_empty(){"local_rules"}else{"windows_description"};
 let known={let db=s.db.lock().unwrap();if let Some(app)=subject.strip_prefix("app:"){features::category_for(&db,app,None)}else{features::category_for(&db,"",subject.strip_prefix("domain:"))}};
 if lookup&&known=="Unclassified"&&infer(&description).is_none(){
  if let Some(domain)=subject.strip_prefix("domain:"){
   let allowed={let db=s.db.lock().unwrap();!excluded(&db,&subject)&&features::read_setting(&db,"activity_data_epoch")==epoch&&reserve_lookup(&db,&subject,now_ms()).unwrap_or(false)};
   if allowed{if let Ok(url)=reqwest::Url::parse(&format!("https://{domain}/")){if let Some(data)=bounded_get(url,65536).await{description=html_description(&String::from_utf8_lossy(&data));source="public_site_description";}}}
  }
  if infer(&description).is_none(){
   let allowed={let db=s.db.lock().unwrap();!excluded(&db,&subject)&&features::read_setting(&db,"activity_data_epoch")==epoch&&!data_lifecycle::is_private(&db,now_ms())&&reserve_lookup(&db,&subject,now_ms()).unwrap_or(false)};
   if allowed{let name=subject.split_once(':').unwrap().1;let mut url=reqwest::Url::parse("https://api.duckduckgo.com/").unwrap();url.query_pairs_mut().append_pair("q",name).append_pair("format","json").append_pair("no_html","1").append_pair("skip_disambig","1");if let Some(data)=bounded_get(url,32768).await{if let Ok(v)=serde_json::from_slice::<Value>(&data){let heading=v["Heading"].as_str().unwrap_or("").to_ascii_lowercase();let identity=name.trim_end_matches(".exe").trim_start_matches("www.");let matched=if subject.starts_with("domain:"){reqwest::Url::parse(v["AbstractURL"].as_str().unwrap_or("")).ok().and_then(|u|u.host_str().map(str::to_string)).is_some_and(|h|h==name||h==identity||h.ends_with(&format!(".{identity}")))}else{identity.len()>=4&&heading.contains(identity)};if matched{description=clean_description(v["AbstractText"].as_str().unwrap_or(""));source="duckduckgo_summary";}}}}
  }
 }
 let db=s.db.lock().unwrap();if features::read_setting(&db,"activity_data_epoch")!=epoch||data_lifecycle::is_private(&db,now_ms())||excluded(&db,&subject){return;}
 let _=apply_evidence(&db,&subject,&description,source,now_ms());
}
fn storage(_:rusqlite::Error)->(StatusCode,String){err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed")}
pub async fn list(State(s):State<Arc<AppState>>)->ApiResult{
 let db=s.db.lock().unwrap();
 let mut q=db.prepare("SELECT subject,category,source,description,reason,llm_approved,prompt_seen_ms,llm_revision FROM service_discovery ORDER BY last_ms DESC LIMIT 100").map_err(storage)?;
 let services=q.query_map([],|r|{
  let subject=r.get::<_,String>(0)?;let current=if let Some(app)=subject.strip_prefix("app:"){features::category_for(&db,app,None)}else{features::category_for(&db,"",subject.strip_prefix("domain:"))};
  let category=if current=="Unclassified"{r.get::<_,String>(1)?}else{current};
  Ok(json!({"subject":subject,"category":category,"source":r.get::<_,String>(2)?,"description":r.get::<_,String>(3)?,"reason":r.get::<_,String>(4)?,"llm_approved":r.get::<_,bool>(5)?&&r.get::<_,String>(7)?==features::read_setting(&db,"classification_config_version"),"seen":r.get::<_,i64>(6)?>0}))
 }).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
 Ok(Json(json!({"services":services,"public_lookup":features::read_setting(&db,"service_public_lookup")=="true","overlay_enabled":features::read_setting(&db,"service_overlay_enabled")!="false","provider":features::read_setting(&db,"classification_provider"),"ai_enabled":features::read_setting(&db,"classification_auto")=="true","privacy_masked":features::read_setting(&db,"presentation_privacy")=="true"||data_lifecycle::is_private(&db,now_ms())})))
}
pub async fn next(State(s):State<Arc<AppState>>)->ApiResult{let db=s.db.lock().unwrap();let hidden=s.paused.load(Ordering::SeqCst)||data_lifecycle::is_private(&db,now_ms())||features::read_setting(&db,"presentation_privacy")=="true"||features::read_setting(&db,"service_overlay_enabled")=="false";
 if hidden{return Ok(Json(json!({"service":null,"hidden":true})));}let last=features::read_setting(&db,"service_last_prompt_ms").parse::<i64>().unwrap_or(0);if now_ms()-last<20000{return Ok(Json(json!({"service":null})));}
 let mut q=db.prepare("SELECT subject,category,source,description FROM service_discovery WHERE checked=1 AND prompt_seen_ms=0 AND last_ms>=?1 ORDER BY last_ms DESC LIMIT 50").map_err(storage)?;let rows=q.query_map(params![now_ms()-300000],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?,r.get::<_,String>(3)?))).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;drop(q);
 for (subject,category,source,description) in rows{if excluded(&db,&subject){continue;}let tx=db.unchecked_transaction().map_err(storage)?;tx.execute("UPDATE service_discovery SET prompt_seen_ms=?2 WHERE subject=?1",params![subject,now_ms()]).map_err(storage)?;tx.execute("INSERT OR REPLACE INTO settings VALUES('service_last_prompt_ms',?1)",params![now_ms().to_string()]).map_err(storage)?;tx.commit().map_err(storage)?;return Ok(Json(json!({"service":{"subject":subject,"category":category,"source":source,"description":description,"ai_enabled":features::read_setting(&db,"classification_auto")=="true","provider":features::read_setting(&db,"classification_provider")},"duration_ms":5500})));}Ok(Json(json!({"service":null}))) }
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Decision{subject:String,action:String,category:Option<String>}
pub async fn decide(State(s):State<Arc<AppState>>,Json(p):Json<Decision>)->ApiResult{let subject=classification_store::canonical(&p.subject).ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_service"))?;let db=s.db.lock().unwrap();if excluded(&db,&subject)||data_lifecycle::is_private(&db,now_ms()){return Err(err(StatusCode::CONFLICT,"service_private_or_excluded"));}let exists:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM service_discovery WHERE subject=?1)",params![subject],|r|r.get(0)).map_err(storage)?;if !exists{return Err(err(StatusCode::NOT_FOUND,"service_not_found"));}let tx=db.unchecked_transaction().map_err(storage)?;
 match p.action.as_str(){"category"=>{let category=p.category.as_deref().filter(|c|features::CATEGORIES.contains(c)).ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_category"))?;tx.execute("INSERT OR REPLACE INTO category_overrides VALUES(?1,?2,?3)",params![subject,category,now_ms()]).map_err(storage)?;tx.execute("UPDATE service_discovery SET category=?2,source='your_choice',llm_approved=0,prompt_seen_ms=?3 WHERE subject=?1",params![subject,category,now_ms()]).map_err(storage)?;},
 "approve_ai"=>{tx.execute("DELETE FROM category_overrides WHERE subject=?1 AND category='Unclassified'",params![subject]).map_err(storage)?;if features::read_setting(&tx,"classification_auto")!="true"{return Err(err(StatusCode::CONFLICT,"configure_classifier_first"));}tx.execute("UPDATE service_discovery SET llm_approved=1,prompt_seen_ms=?2,llm_revision=?3 WHERE subject=?1",params![subject,now_ms(),features::read_setting(&tx,"classification_config_version")]).map_err(storage)?;tx.execute("INSERT INTO classifications(subject,state,category,reason,created_ms,updated_ms) VALUES(?1,'awaiting_consent','Unclassified','Owner approved AI for this service',?2,?2) ON CONFLICT(subject) DO UPDATE SET state=CASE WHEN state IN('unresolved','succeeded') THEN 'awaiting_consent' ELSE state END",params![subject,now_ms()]).map_err(storage)?;},
 "later"=>{tx.execute("UPDATE service_discovery SET prompt_seen_ms=?2 WHERE subject=?1",params![subject,now_ms()]).map_err(storage)?;},_=>return Err(err(StatusCode::BAD_REQUEST,"invalid_service_action"))}tx.commit().map_err(storage)?;drop(db);if p.action=="approve_ai"{tokio::spawn(classification::worker(Arc::clone(&s)));}Ok(Json(json!({"ok":true}))) }
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Preferences{public_lookup:bool,overlay_enabled:bool}
pub async fn preferences(State(s):State<Arc<AppState>>,Json(p):Json<Preferences>)->ApiResult{let db=s.db.lock().unwrap();let tx=db.unchecked_transaction().map_err(storage)?;data_lifecycle::advance_epoch(&tx).map_err(storage)?;if p.public_lookup&&features::read_setting(&tx,"service_public_lookup")!="true"{tx.execute("UPDATE service_discovery SET checked=0 WHERE category='Unclassified' AND llm_approved=0",[]).map_err(storage)?;}tx.execute("INSERT OR REPLACE INTO settings VALUES('service_public_lookup',?1),('service_overlay_enabled',?2)",params![p.public_lookup.to_string(),p.overlay_enabled.to_string()]).map_err(storage)?;tx.commit().map_err(storage)?;Ok(Json(json!({"ok":true}))) }
#[cfg(test)]mod tests{use super::*;
 #[test]fn description_rules_are_small_conservative_and_do_not_read_prompts(){assert_eq!(infer("Local-first AI Development Environment"),Some("Development"));assert_eq!(infer("A video editor for creators"),Some("Design"));assert_eq!(infer("AI for everything"),None);assert_eq!(infer("education and development"),None);assert!(clean_description("C:\\Users\\owner\\secret").is_empty());}
 #[test]fn metadata_parser_uses_only_bounded_description_tags(){assert_eq!(html_description("<title>private title</title><meta content='AI Development Environment' name='description'>"),"AI Development Environment");assert!(html_description("<script>description='private'</script><title>code editor</title>").is_empty());}
 #[test]fn public_lookup_rejects_local_and_mapped_addresses(){for ip in ["127.0.0.1","10.0.0.1","169.254.169.254","100.64.0.1","::1","fc00::1","::ffff:127.0.0.1"]{assert!(!public_ip(ip.parse().unwrap()));}assert!(public_ip("8.8.8.8".parse().unwrap()));}
 #[test]fn local_description_is_remembered_and_manual_choice_wins_without_ai_budget(){
  let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();db.execute("INSERT INTO service_discovery(subject,first_ms,last_ms) VALUES('app:hive.exe',1,1)",[]).unwrap();
  apply_evidence(&db,"app:hive.exe","Local-first AI Development Environment","windows_description",1000).unwrap();assert_eq!(features::category_for(&db,"hive.exe",None),"Development");assert!(classification_store::claim(&db,2000,true,20).unwrap().is_none());
  db.execute("INSERT INTO category_overrides VALUES('app:hive.exe','Education',3000)",[]).unwrap();apply_evidence(&db,"app:hive.exe","AI Development Environment","windows_description",4000).unwrap();assert_eq!(features::category_for(&db,"hive.exe",None),"Education");
  assert_eq!(db.query_row("SELECT COUNT(*) FROM classification_requests",[],|r|r.get::<_,i64>(0)).unwrap(),0);
 }
 #[test]fn public_lookup_budget_and_scoped_deletion_cover_discovery_metadata(){
  let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();for _ in 0..20{assert!(reserve_lookup(&db,"domain:example.com",1000).unwrap());}assert!(!reserve_lookup(&db,"domain:example.com",2000).unwrap());
  db.execute("INSERT INTO service_discovery(subject,first_ms,last_ms,description) VALUES('domain:example.com',1,1,'public description')",[]).unwrap();data_lifecycle::erase_subjects(&db,&["domain:example.com".into()],3000).unwrap();assert_eq!(db.query_row("SELECT COUNT(*) FROM service_discovery",[],|r|r.get::<_,i64>(0)).unwrap(),0);assert_eq!(db.query_row("SELECT COUNT(*) FROM service_lookup_requests",[],|r|r.get::<_,i64>(0)).unwrap(),0);
 }
}

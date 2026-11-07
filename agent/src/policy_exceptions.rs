//! Exceptions override one selected rule/target. Other restrictions continue.
use crate::{AppState,ApiResult,err,now_ms,domain};
use axum::{extract::{State,Path},http::StatusCode,Json};
use rusqlite::{params,Connection,OptionalExtension};use serde::Deserialize;use serde_json::{json,Value};use std::sync::Arc;
pub fn migrate(db:&Connection)->rusqlite::Result<()>{db.execute_batch("CREATE TABLE IF NOT EXISTS temporary_exceptions(id TEXT PRIMARY KEY,rule_id TEXT NOT NULL,domain TEXT NOT NULL,profile_id TEXT,created_ms INTEGER NOT NULL,expires_ms INTEGER NOT NULL,reason TEXT NOT NULL,revoked INTEGER NOT NULL DEFAULT 0);
 CREATE INDEX IF NOT EXISTS idx_exceptions_active ON temporary_exceptions(rule_id,expires_ms,revoked);
 CREATE TABLE IF NOT EXISTS learning_sessions(focus_id TEXT PRIMARY KEY REFERENCES focus_sessions(id) ON DELETE CASCADE,video_ids_json TEXT NOT NULL,rule_ids_json TEXT NOT NULL,hide_home INTEGER NOT NULL,hide_related INTEGER NOT NULL,hide_shorts INTEGER NOT NULL,stop_autoplay INTEGER NOT NULL);")}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct NewException{rule_id:String,domain:String,minutes:u32,reason:Option<String>,profile_id:Option<String>}
pub async fn list(State(s):State<Arc<AppState>>)->ApiResult{
 let db=s.db.lock().unwrap();let mut q=db.prepare("SELECT e.id,e.rule_id,e.domain,e.profile_id,e.created_ms,e.expires_ms,e.reason,e.revoked,COALESCE(r.name,'Focus session') FROM temporary_exceptions e LEFT JOIN rules r ON r.id=e.rule_id WHERE expires_ms>?1 AND revoked=0 ORDER BY created_ms DESC LIMIT 100").map_err(storage)?;
 let entries=q.query_map(params![now_ms()],|r|Ok(json!({"id":r.get::<_,String>(0)?,"rule_id":r.get::<_,String>(1)?,"domain":r.get::<_,String>(2)?,"profile_id":r.get::<_,Option<String>>(3)?,"created_ms":r.get::<_,i64>(4)?,"expires_ms":r.get::<_,i64>(5)?,"reason":r.get::<_,String>(6)?,"revoked":r.get::<_,i64>(7)?!=0,"rule_name":r.get::<_,String>(8)?}))).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
 Ok(Json(json!({"ok":true,"exceptions":entries,"scope":"one selected rule and exact configured domain; other rules still apply","emergency_recovery_separate":true})))
}
pub async fn create(State(s):State<Arc<AppState>>,Json(p):Json<NewException>)->ApiResult{
 if ![2,5,10].contains(&p.minutes)||uuid::Uuid::parse_str(&p.rule_id).is_err()||!domain::valid_domain(&p.domain)||p.profile_id.as_ref().is_some_and(|v|!crate::tracking::valid_profile(v))||p.reason.as_deref().is_some_and(|r|!["planned_use","study","work","break"].contains(&r)){return Err(err(StatusCode::BAD_REQUEST,"invalid_temporary_exception"));}
 let db=s.db.lock().unwrap();let tx=db.unchecked_transaction().map_err(storage)?;let now=now_ms();
 let rule=tx.query_row("SELECT domain FROM rules WHERE id=?1 AND enabled=1",params![p.rule_id],|r|r.get::<_,String>(0)).optional().map_err(storage)?;
 let valid=if let Some(domain)=rule{domain==p.domain}else{crate::features::active_focus(&tx,now).map_err(storage)?.is_some_and(|f|f["id"]==p.rule_id&&f["block_domains"].as_array().is_some_and(|a|a.iter().any(|d|d==&json!(p.domain))))};
 if !valid{return Err(err(StatusCode::BAD_REQUEST,"exception_target_must_match_selected_rule"));}
 if tx.query_row("SELECT COUNT(*) FROM temporary_exceptions WHERE revoked=0 AND expires_ms>?1",params![now],|r|r.get::<_,i64>(0)).map_err(storage)?>=100{return Err(err(StatusCode::CONFLICT,"too_many_active_exceptions"));}
 tx.execute("UPDATE temporary_exceptions SET revoked=1 WHERE rule_id=?1 AND domain=?2 AND profile_id IS ?3 AND revoked=0",params![p.rule_id,p.domain,p.profile_id]).map_err(storage)?;
 let id=uuid::Uuid::new_v4().to_string();let end=now+p.minutes as i64*60000;
 tx.execute("INSERT INTO temporary_exceptions(id,rule_id,domain,profile_id,created_ms,expires_ms,reason) VALUES(?1,?2,?3,?4,?5,?6,?7)",params![id,p.rule_id,p.domain,p.profile_id,now,end,p.reason.unwrap_or_else(||"planned_use".into())]).map_err(storage)?;
 tx.commit().map_err(storage)?;Ok(Json(json!({"ok":true,"id":id,"expires_ms":end,"other_rules_still_apply":true})))
}
pub async fn revoke(Path(id):Path<String>,State(s):State<Arc<AppState>>)->ApiResult{if uuid::Uuid::parse_str(&id).is_err(){return Err(err(StatusCode::BAD_REQUEST,"invalid_id"));}s.db.lock().unwrap().execute("UPDATE temporary_exceptions SET revoked=1 WHERE id=?1",params![id]).map_err(storage)?;Ok(Json(json!({"ok":true})))}
#[derive(Clone,Deserialize)]#[serde(deny_unknown_fields)]pub struct LearningInput{pub video_ids:Vec<String>,pub override_rule_ids:Vec<String>,pub hide_home:bool,pub hide_related:bool,pub hide_shorts:bool,pub stop_autoplay:bool}
impl LearningInput{pub fn valid(&self)->bool{!self.video_ids.is_empty()&&self.video_ids.len()<=50&&self.video_ids.iter().all(|id|(5..=128).contains(&id.len())&&id.bytes().all(|b|b.is_ascii_alphanumeric()||b"_-".contains(&b)))&&self.override_rule_ids.len()<=20&&self.override_rule_ids.iter().all(|id|id=="focus"||uuid::Uuid::parse_str(id).is_ok())}}
pub fn start_learning(db:&Connection,focus:&str,p:&LearningInput)->ApiResult{
 if !p.valid(){return Err(err(StatusCode::BAD_REQUEST,"invalid_learning_scope"));}
 let mut rules=Vec::new();for id in &p.override_rule_ids{
  if id=="focus"{rules.push(focus.to_owned());continue;}
  let domain=db.query_row("SELECT domain FROM rules WHERE id=?1 AND enabled=1",params![id],|r|r.get::<_,String>(0)).optional().map_err(storage)?;
  if !domain.is_some_and(|d|["youtube.com","www.youtube.com"].contains(&d.as_str())){return Err(err(StatusCode::BAD_REQUEST,"learning_override_requires_selected_youtube_rule"));}rules.push(id.clone());
 }
 db.execute("INSERT INTO learning_sessions VALUES(?1,?2,?3,?4,?5,?6,?7)",params![focus,serde_json::to_string(&p.video_ids).unwrap(),serde_json::to_string(&rules).unwrap(),p.hide_home as u8,p.hide_related as u8,p.hide_shorts as u8,p.stop_autoplay as u8]).map_err(storage)?;Ok(Json(json!({"ok":true})))
}
pub fn apply(db:&Connection,blocks:Vec<Value>,profile:Option<&str>,now:i64)->rusqlite::Result<(Vec<Value>,Option<i64>)>{
 let mut q=db.prepare("SELECT rule_id,domain,expires_ms FROM temporary_exceptions WHERE revoked=0 AND created_ms<=?1 AND expires_ms>?1 AND (profile_id IS NULL OR profile_id=?2)")?;
 let exceptions=q.query_map(params![now,profile],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,i64>(2)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
 let next=exceptions.iter().map(|e|e.2).min();let mut blocks=blocks.into_iter().filter(|b|!exceptions.iter().any(|(id,domain,_)|b["id"]==*id&&b["domain"]==*domain)).collect::<Vec<_>>();
 if let Some(focus)=crate::features::active_focus(db,now)?.filter(|f|f["cycle"]["state"]!="paused"){
  let id=focus["id"].as_str().unwrap_or("");let learning=db.query_row("SELECT video_ids_json,rule_ids_json,hide_home,hide_related,hide_shorts,stop_autoplay FROM learning_sessions WHERE focus_id=?1",params![id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,bool>(2)?,r.get::<_,bool>(3)?,r.get::<_,bool>(4)?,r.get::<_,bool>(5)?))).optional()?;
  if let Some((videos,rules,home,related,shorts,autoplay))=learning{
   let videos:Value=serde_json::from_str(&videos).unwrap_or(json!([]));let rules:Vec<String>=serde_json::from_str(&rules).unwrap_or_default();
   for b in &mut blocks{if b["id"].as_str().is_some_and(|id|rules.iter().any(|r|r==id))&&b["domain"].as_str().is_some_and(|d|["youtube.com","www.youtube.com"].contains(&d)){b["allow_video_ids"]=videos.clone();}}
   let end=focus["end_ms"].as_i64().unwrap_or(now).min(now+180000);
   let parts=[("home",home),("related",related),("shorts",shorts)].into_iter().filter(|(_,on)|*on).map(|(p,_)|p).collect::<Vec<_>>();
   if !parts.is_empty(){blocks.push(json!({"id":format!("{id}-learning-feed"),"domain":"www.youtube.com","path_prefix":null,"action":"hide_feed","feed_parts":parts,"reason":"Selected Learning Mode controls","expires_at_ms":end}));}
   if shorts{blocks.push(json!({"id":format!("{id}-learning-shorts"),"domain":"www.youtube.com","path_prefix":"/shorts/","action":"block","reason":"Learning Mode Shorts routes","expires_at_ms":end}));}
   if autoplay{blocks.push(json!({"id":format!("{id}-learning-autoplay"),"domain":"www.youtube.com","path_prefix":null,"action":"stop_autoplay","reason":"Learning Mode autoplay","expires_at_ms":end}));}
  }
 }
 Ok((blocks,next))
}
fn storage(_:rusqlite::Error)->(StatusCode,String){err(StatusCode::INTERNAL_SERVER_ERROR,"exception_storage_failed")}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn normal_exception_only_releases_the_selected_rule_domain_and_profile_until_expiry(){
  let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();db.execute("INSERT INTO temporary_exceptions VALUES('id','selected','www.youtube.com','profile',100,200,'study',0)",[]).unwrap();
  let blocks=vec![json!({"id":"selected","domain":"www.youtube.com"}),json!({"id":"other-rule","domain":"www.youtube.com"}),json!({"id":"selected","domain":"www.instagram.com"})];
  assert_eq!(apply(&db,blocks.clone(),Some("profile"),150).unwrap().0.len(),2);assert_eq!(apply(&db,blocks.clone(),Some("other"),150).unwrap().0.len(),3);assert_eq!(apply(&db,blocks,Some("profile"),201).unwrap().0.len(),3);
 }
}

//! Browser/profile identity and the end-to-end Tracking Doctor.
use crate::{AppState,ApiResult,err,now_ms};
use axum::{extract::State,http::StatusCode,Json};
use rusqlite::{params,Connection};
use serde::Deserialize;
use serde_json::{json,Value};
use std::sync::Arc;

pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS browser_instances(
  extension_id TEXT NOT NULL,profile_id TEXT NOT NULL,browser TEXT NOT NULL,first_seen_ms INTEGER NOT NULL,last_seen_ms INTEGER NOT NULL,
  recording INTEGER,queued INTEGER NOT NULL DEFAULT 0,adapter TEXT NOT NULL DEFAULT 'unknown',qualified_at_ms INTEGER NOT NULL DEFAULT 0,
  accepted_at_ms INTEGER NOT NULL DEFAULT 0,accepted_count INTEGER NOT NULL DEFAULT 0,rendered_ms INTEGER NOT NULL DEFAULT 0,
  policy_ack_ms INTEGER NOT NULL DEFAULT 0,domain TEXT,focused INTEGER NOT NULL DEFAULT 0,last_activity_ms INTEGER NOT NULL DEFAULT 0,
  companion_version TEXT NOT NULL DEFAULT '',loss_count INTEGER NOT NULL DEFAULT 0,revoked INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY(extension_id,profile_id));
 CREATE INDEX IF NOT EXISTS idx_browser_foreground ON browser_instances(browser,focused,last_activity_ms);
 CREATE TABLE IF NOT EXISTS video_visits(event_id TEXT PRIMARY KEY,platform TEXT NOT NULL,video_id TEXT NOT NULL,occurred_ms INTEGER NOT NULL,profile_id TEXT,extension_id TEXT);
 CREATE INDEX IF NOT EXISTS idx_video_visits_time ON video_visits(occurred_ms);")?;
 for (table,column) in [("videos","profile_id TEXT"),("videos","extension_id TEXT"),("raw_events","profile_id TEXT"),("raw_events","extension_id TEXT")] {
  let name=column.split(' ').next().unwrap();
  let exists={let mut q=db.prepare(&format!("PRAGMA table_info({table})"))?;let names=q.query_map([],|r|r.get::<_,String>(1))?.collect::<rusqlite::Result<Vec<_>>>()?;names.iter().any(|n|n==name)};
  if !exists{db.execute_batch(&format!("ALTER TABLE {table} ADD COLUMN {column};"))?;}
 }
 Ok(())
}
#[derive(Default,Deserialize)]#[serde(deny_unknown_fields)]
pub struct SyncInfo {
 pub profile_id:Option<String>,pub browser:Option<String>,pub recording_enabled:Option<bool>,pub queued:Option<u32>,
 pub adapter:Option<String>,pub qualified_at_ms:Option<i64>,pub companion_version:Option<String>,pub loss_count:Option<u64>
}
pub fn valid_profile(id:&str)->bool{uuid::Uuid::parse_str(id).is_ok()&&id.len()==36}
fn ensure_instance(db:&Connection,ext:&str,profile:&str,browser:&str,at:i64)->rusqlite::Result<bool>{
 let exists=db.query_row("SELECT EXISTS(SELECT 1 FROM browser_instances WHERE extension_id=?1 AND profile_id=?2)",params![ext,profile],|r|r.get::<_,bool>(0))?;
 if !exists&&db.query_row("SELECT COUNT(*) FROM browser_instances WHERE extension_id=?1",params![ext],|r|r.get::<_,i64>(0))?>=50{return Err(rusqlite::Error::InvalidQuery);}
 db.execute("INSERT OR IGNORE INTO browser_instances(extension_id,profile_id,browser,first_seen_ms,last_seen_ms) VALUES(?1,?2,?3,?4,?4)",params![ext,profile,browser,at])?;
 Ok(db.query_row("SELECT revoked=0 FROM browser_instances WHERE extension_id=?1 AND profile_id=?2",params![ext,profile],|r|r.get(0))?)
}
pub fn sync(db:&Connection,ext:&str,enrolled_browser:&str,p:SyncInfo,at:i64)->Result<(),&'static str>{
 let Some(profile)=p.profile_id else{return Ok(());};
 if !valid_profile(&profile)||p.browser.as_ref().is_some_and(|b|b!=&enrolled_browser.to_ascii_lowercase())||p.queued.is_some_and(|n|n>2000)||
  p.adapter.as_deref().is_some_and(|a|!["ready","unsupported","unknown","recording_off","private_session","no_player"].contains(&a))||p.companion_version.as_ref().is_some_and(|v|v.len()>30||!v.bytes().all(|b|b.is_ascii_digit()||b==b'.'))||p.qualified_at_ms.is_some_and(|t|t<0||t>at+300000){return Err("invalid_profile_status");}
 if !ensure_instance(db,ext,&profile,&enrolled_browser.to_ascii_lowercase(),at).map_err(|_|"storage_failed")?{return Err("profile_revoked_authorize_in_desktop");}
 db.execute("UPDATE browser_instances SET last_seen_ms=?3,recording=COALESCE(?4,recording),queued=COALESCE(?5,queued),adapter=COALESCE(?6,adapter),
  qualified_at_ms=max(qualified_at_ms,COALESCE(?7,0)),companion_version=COALESCE(?8,companion_version),loss_count=COALESCE(?9,loss_count),
  domain=CASE WHEN ?4=0 THEN NULL ELSE domain END,focused=CASE WHEN ?4=0 THEN 0 ELSE focused END
  WHERE extension_id=?1 AND profile_id=?2",params![ext,profile,at,p.recording_enabled.map(|v|v as u8),p.queued,p.adapter,p.qualified_at_ms,p.companion_version,p.loss_count.map(|v|v.min(i64::MAX as u64) as i64)]).map_err(|_|"storage_failed")?;
 Ok(())
}
pub fn record_event(db:&Connection,ext:Option<&str>,profile:Option<&str>,browser:Option<&str>,domain:Option<&str>,focused:bool,occurred:i64,qualified:bool,ack:bool)->rusqlite::Result<()>{
 let (Some(ext),Some(profile))=(ext,profile)else{return Ok(());};
 let kind=browser.map(str::to_owned).or_else(||db.query_row("SELECT lower(browser) FROM browser_pairs WHERE extension_id=?1",params![ext],|r|r.get(0)).ok()).unwrap_or_default();
 if !ensure_instance(db,ext,profile,&kind,now_ms())?{return Err(rusqlite::Error::InvalidQuery);}
 db.execute("UPDATE browser_instances SET last_seen_ms=?3,accepted_at_ms=CASE WHEN ?4=1 THEN ?3 ELSE accepted_at_ms END,accepted_count=accepted_count+1,
  qualified_at_ms=CASE WHEN ?4=1 THEN max(qualified_at_ms,?5) ELSE qualified_at_ms END,
  policy_ack_ms=CASE WHEN ?6=1 THEN ?3 ELSE policy_ack_ms END,
  domain=CASE WHEN ?7=1 AND ?5>=last_activity_ms THEN ?8 ELSE domain END,
  focused=CASE WHEN ?7=1 AND ?5>=last_activity_ms THEN ?9 ELSE focused END,
  last_activity_ms=CASE WHEN ?7=1 THEN max(last_activity_ms,?5) ELSE last_activity_ms END
  WHERE extension_id=?1 AND profile_id=?2",params![ext,profile,now_ms(),qualified as u8,occurred,ack as u8,browser.is_some() as u8,domain,focused as u8])?;
 Ok(())
}
pub fn foreground(db:&Connection,app:&str,at:i64)->rusqlite::Result<Option<(Option<String>,String)>>{
 let browser=match app.to_ascii_lowercase().as_str(){"brave.exe"=>"brave","chrome.exe"=>"chrome","msedge.exe"=>"edge",_=>return Ok(None)};
 let mut q=db.prepare("SELECT domain,profile_id FROM browser_instances WHERE browser=?1 AND revoked=0 AND focused=1 AND (recording IS NULL OR recording=1) AND last_activity_ms>=?2 AND last_activity_ms<=?3 LIMIT 2")?;
 let rows=q.query_map(params![browser,at-30000,at+5000],|r|Ok((r.get::<_,Option<String>>(0)?,r.get::<_,String>(1)?)))?.collect::<rusqlite::Result<Vec<_>>>()?;
 Ok(if rows.len()==1{rows.into_iter().next()}else{None})
}
pub fn receipts(db:&Connection)->rusqlite::Result<Vec<Value>>{
 let mut q=db.prepare("SELECT extension_id,profile_id,accepted_at_ms FROM browser_instances WHERE revoked=0 AND accepted_at_ms>0 ORDER BY accepted_at_ms DESC LIMIT 50")?;
 let rows=q.query_map([],|r|Ok(json!({"extension_id":r.get::<_,String>(0)?,"profile_id":r.get::<_,String>(1)?,"accepted_at_ms":r.get::<_,i64>(2)?})))?.collect::<rusqlite::Result<Vec<_>>>()?;Ok(rows)
}
pub async fn doctor(State(s):State<Arc<AppState>>)->ApiResult{
 let db=s.db.lock().unwrap();let now=now_ms();
 let mut q=db.prepare("SELECT i.extension_id,i.profile_id,i.browser,i.last_seen_ms,i.recording,i.queued,i.adapter,i.qualified_at_ms,i.accepted_at_ms,i.rendered_ms,i.policy_ack_ms,i.companion_version,i.loss_count,i.revoked,i.accepted_count FROM browser_instances i JOIN browser_pairs p ON p.extension_id=i.extension_id ORDER BY i.last_seen_ms DESC LIMIT 100").map_err(storage)?;
 let profiles=q.query_map([],|r|{let seen=r.get::<_,i64>(3)?;let recording=r.get::<_,Option<i64>>(4)?;let qualified=r.get::<_,i64>(7)?;let accepted=r.get::<_,i64>(8)?;let rendered=r.get::<_,i64>(9)?;let ack=r.get::<_,i64>(10)?;let revoked=r.get::<_,i64>(13)?!=0;
  Ok(json!({"extension_id":r.get::<_,String>(0)?,"profile_id":r.get::<_,String>(1)?,"browser":r.get::<_,String>(2)?,"last_seen_ms":seen,"recording_enabled":recording.map(|v|v!=0),"queued":r.get::<_,i64>(5)?,"adapter":r.get::<_,String>(6)?,"qualified_at_ms":qualified,"accepted_at_ms":accepted,"rendered_ms":rendered,"policy_ack_ms":ack,"companion_version":r.get::<_,String>(11)?,"loss_count":r.get::<_,i64>(12)?,"revoked":revoked,"accepted_events":r.get::<_,i64>(14)?,
   "online":now-seen<90000,"authorized":!revoked,"dashboard_received":accepted>0&&rendered>=accepted,"policy_acknowledged":ack>now-180000}))
 }).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
 Ok(Json(json!({"ok":true,"checked_at_ms":now,"profiles":profiles,"agent_version":env!("CARGO_PKG_VERSION"),"expected_companion_version":"0.3.8","protocol":"1.0 additive profile identity","play_threshold_ms":2000,"native_attribution":"Conservative matching of browser-reported focus; ambiguous profiles remain Unattributed"})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Receipt{extension_id:String,profile_id:String,accepted_at_ms:i64}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct RenderAck{receipts:Vec<Receipt>}
pub async fn rendered(State(s):State<Arc<AppState>>,Json(p):Json<RenderAck>)->ApiResult{
 if p.receipts.len()>50||p.receipts.iter().any(|r|!valid_profile(&r.profile_id)||r.extension_id.len()!=32||!r.extension_id.bytes().all(|b|(b'a'..=b'p').contains(&b))||r.accepted_at_ms<0||r.accepted_at_ms>now_ms()+5000){return Err(err(StatusCode::BAD_REQUEST,"invalid_render_receipt"));}
 let db=s.db.lock().unwrap();let tx=db.unchecked_transaction().map_err(storage)?;
 for r in p.receipts{tx.execute("UPDATE browser_instances SET rendered_ms=max(rendered_ms,min(?3,accepted_at_ms)) WHERE extension_id=?1 AND profile_id=?2",params![r.extension_id,r.profile_id,r.accepted_at_ms]).map_err(storage)?;}
 tx.commit().map_err(storage)?;Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct ProfileAccess{extension_id:String,profile_id:String,authorized:bool}
pub async fn access(State(s):State<Arc<AppState>>,Json(p):Json<ProfileAccess>)->ApiResult{
 if !valid_profile(&p.profile_id)||p.extension_id.len()!=32||!p.extension_id.bytes().all(|b|(b'a'..=b'p').contains(&b)){return Err(err(StatusCode::BAD_REQUEST,"invalid_profile"));}
 let changed=s.db.lock().unwrap().execute("UPDATE browser_instances SET revoked=?3,domain=NULL,focused=0 WHERE extension_id=?1 AND profile_id=?2",params![p.extension_id,p.profile_id,(!p.authorized) as u8]).map_err(storage)?;
 if changed==0{return Err(err(StatusCode::NOT_FOUND,"profile_not_found"));}Ok(Json(json!({"ok":true})))
}
fn storage(_:rusqlite::Error)->(StatusCode,String){err(StatusCode::INTERNAL_SERVER_ERROR,"tracking_storage_failed")}
#[cfg(test)]mod tests{
 use super::*;
 fn db()->Connection{let d=Connection::open_in_memory().unwrap();d.execute_batch("CREATE TABLE videos(instance_id TEXT);CREATE TABLE raw_events(event_id TEXT);CREATE TABLE browser_pairs(extension_id TEXT,browser TEXT);").unwrap();migrate(&d).unwrap();d}
 #[test]fn profiles_do_not_overwrite_or_borrow_each_others_domain(){let d=db();let p1=uuid::Uuid::new_v4().to_string();let p2=uuid::Uuid::new_v4().to_string();let ext="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";let at=now_ms();record_event(&d,Some(ext),Some(&p1),Some("brave"),Some("one.example"),true,at,false,false).unwrap();assert_eq!(foreground(&d,"brave.exe",at).unwrap().unwrap().0.as_deref(),Some("one.example"));assert!(foreground(&d,"chrome.exe",at).unwrap().is_none());record_event(&d,Some(ext),Some(&p2),Some("brave"),Some("two.example"),true,at,false,false).unwrap();assert!(foreground(&d,"brave.exe",at).unwrap().is_none());assert!(foreground(&d,"brave.exe",at+40000).unwrap().is_none());}
 #[test]fn recording_off_removes_foreground_details_but_keeps_authorization(){let d=db();let p=uuid::Uuid::new_v4().to_string();let ext="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";let at=now_ms();record_event(&d,Some(ext),Some(&p),Some("brave"),Some("one.example"),true,at,false,false).unwrap();sync(&d,ext,"Brave",SyncInfo{profile_id:Some(p),browser:Some("brave".into()),recording_enabled:Some(false),..Default::default()},at).unwrap();assert!(foreground(&d,"brave.exe",at).unwrap().is_none());}
}

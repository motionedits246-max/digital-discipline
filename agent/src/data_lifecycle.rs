//! Prospective privacy windows, scoped history erasure and stale-delivery tombstones.
use crate::{AppState,ApiResult,err,now_ms,features,classification_store};
use rusqlite::{Connection,params};
use axum::{extract::State,http::StatusCode,Json};
use serde::Deserialize;use serde_json::{json,Value};use std::sync::Arc;
pub fn migrate(db:&Connection)->rusqlite::Result<()>{db.execute_batch("CREATE TABLE IF NOT EXISTS purged_subjects(subject TEXT PRIMARY KEY,before_ms INTEGER NOT NULL);
 CREATE TABLE IF NOT EXISTS private_windows(id TEXT PRIMARY KEY,start_ms INTEGER NOT NULL,end_ms INTEGER NOT NULL);
 CREATE INDEX IF NOT EXISTS idx_private_windows_range ON private_windows(start_ms,end_ms);
 INSERT OR IGNORE INTO settings(key,value) VALUES('activity_data_epoch','initial');
 INSERT OR IGNORE INTO settings(key,value) VALUES('activity_deleted_before_ms','0');
 INSERT OR IGNORE INTO settings(key,value) VALUES('private_until_ms','0');")}
pub fn is_private(db:&Connection,at:i64)->bool{features::read_setting(db,"private_until_ms").parse::<i64>().unwrap_or(0)>at}
pub fn private_info(db:&Connection,at:i64)->Value{let until=features::read_setting(db,"private_until_ms").parse::<i64>().unwrap_or(0);json!({"active":until>at,"until_ms":if until>at{Some(until)}else{None},"automatic_ai_paused":until>at,"restrictions_continue":true})}
pub fn advance_epoch(db:&Connection)->rusqlite::Result<()>{db.execute("INSERT OR REPLACE INTO settings(key,value) VALUES('activity_data_epoch',?1)",params![uuid::Uuid::new_v4().to_string()])?;Ok(())}
pub fn rejection(db:&Connection,occurred:i64,subject:Option<&str>)->rusqlite::Result<Option<&'static str>>{
 let cutoff=features::read_setting(db,"activity_deleted_before_ms").parse::<i64>().unwrap_or(0);
 if occurred<=cutoff{return Ok(Some("ignored_deleted"));}
 let private:bool=db.query_row("SELECT EXISTS(SELECT 1 FROM private_windows WHERE start_ms<=?1 AND end_ms>=?1)",params![occurred],|r|r.get(0))?;
 if private{return Ok(Some("ignored_private"));}
 if let Some(subject)=subject{
  let mut q=db.prepare("SELECT subject FROM purged_subjects WHERE before_ms>=?1")?;
  for candidate in q.query_map(params![occurred],|r|r.get::<_,String>(0))?{let scope=candidate?;if subject==scope||scope.starts_with("domain:")&&subject.starts_with("domain:")&&subject.ends_with(&format!(".{}",&scope[7..])){return Ok(Some("ignored_deleted"));}}
 }
 Ok(None)
}
pub fn clear_activity(db:&Connection,at:i64)->rusqlite::Result<()>{
 let tx=db.unchecked_transaction()?;
 tx.execute_batch("DELETE FROM sessions;DELETE FROM raw_events;DELETE FROM videos;DELETE FROM video_visits;
  DELETE FROM classifications;DELETE FROM classification_memory;DELETE FROM classification_job_state;DELETE FROM classification_attempts;DELETE FROM classification_requests;
  DELETE FROM local_inbox;DELETE FROM audits;DELETE FROM rule_states;DELETE FROM focus_snoozes;DELETE FROM focus_sessions;DELETE FROM device_sessions;DELETE FROM linked_devices;
  UPDATE browser_instances SET domain=NULL,focused=0,qualified_at_ms=0,accepted_at_ms=0,accepted_count=0,rendered_ms=0,last_activity_ms=0;")?;
 tx.execute("INSERT OR REPLACE INTO settings(key,value) VALUES('activity_deleted_before_ms',?1)",params![at.to_string()])?;
 advance_epoch(&tx)?;tx.commit()
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct PurgeInput{subjects:Vec<String>}
pub fn erase_subjects(db:&Connection,subjects:&[String],at:i64)->rusqlite::Result<()>{
 for subject in subjects{
  let app=subject.strip_prefix("app:");let host=subject.strip_prefix("domain:");
  if let Some(app)=app{db.execute("DELETE FROM sessions WHERE lower(app)=?1",params![app])?;db.execute("DELETE FROM device_sessions WHERE lower(app)=?1",params![app])?;}
  if let Some(host)=host{
   db.execute("DELETE FROM sessions WHERE domain=?1 OR domain LIKE ?2",params![host,format!("%.{host}")])?;
   db.execute("DELETE FROM device_sessions WHERE domain=?1 OR domain LIKE ?2",params![host,format!("%.{host}")])?;
   let platform=if host=="youtube.com"||host=="www.youtube.com"{Some("youtube_%")}else if host=="instagram.com"||host=="www.instagram.com"{Some("instagram_reels")}else{None};
   if let Some(platform)=platform{db.execute("DELETE FROM raw_events WHERE event_id IN(SELECT event_id FROM video_visits WHERE platform LIKE ?1)",params![platform])?;db.execute("DELETE FROM videos WHERE platform LIKE ?1",params![platform])?;db.execute("DELETE FROM video_visits WHERE platform LIKE ?1",params![platform])?;}
   db.execute("UPDATE browser_instances SET domain=NULL,focused=0 WHERE domain=?1 OR domain LIKE ?2",params![host,format!("%.{host}")])?;
  }
  db.execute("DELETE FROM local_inbox WHERE subject=?1 OR (?2 IS NOT NULL AND subject LIKE ?3)",params![subject,host,format!("domain:%.{}",host.unwrap_or(""))])?;
  for table in ["classifications","classification_memory","classification_job_state","classification_attempts"]{
   db.execute(&format!("DELETE FROM {table} WHERE lower(subject)=?1 OR (?2 IS NOT NULL AND subject LIKE ?3)"),params![subject,host,format!("domain:%.{}",host.unwrap_or(""))])?;
  }
  db.execute("INSERT INTO purged_subjects(subject,before_ms) VALUES(?1,?2) ON CONFLICT(subject) DO UPDATE SET before_ms=max(before_ms,excluded.before_ms)",params![subject,at])?;
 }
 advance_epoch(db)
}
pub async fn purge(State(s):State<Arc<AppState>>,Json(p):Json<PurgeInput>)->ApiResult{
 if p.subjects.is_empty()||p.subjects.len()>50{return Err(err(StatusCode::BAD_REQUEST,"invalid_purge_scope"));}
 let subjects=p.subjects.iter().map(|x|classification_store::canonical(x)).collect::<Option<Vec<_>>>().ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_purge_scope"))?;
 let mut tracker=s.tracker.lock().unwrap();tracker.reconciler.stop();tracker.session_id=None;let db=s.db.lock().unwrap();let tx=db.unchecked_transaction().map_err(storage)?;erase_subjects(&tx,&subjects,now_ms()).map_err(storage)?;tx.commit().map_err(storage)?;
 Ok(Json(json!({"ok":true,"subjects":subjects,"rules_and_overrides_retained":true,"future_collection_unchanged":true})))
}
pub async fn private_status(State(s):State<Arc<AppState>>)->ApiResult{let db=s.db.lock().unwrap();Ok(Json(private_info(&db,now_ms())))}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct PrivateInput{minutes:u32}
pub async fn private_session(State(s):State<Arc<AppState>>,Json(p):Json<PrivateInput>)->ApiResult{
 if p.minutes>1440{return Err(err(StatusCode::BAD_REQUEST,"invalid_private_duration"));}
 let mut tracker=s.tracker.lock().unwrap();tracker.reconciler.stop();tracker.session_id=None;let db=s.db.lock().unwrap();let tx=db.unchecked_transaction().map_err(storage)?;let now=now_ms();let until=now+p.minutes as i64*60000;
 tx.execute("UPDATE private_windows SET end_ms=?1 WHERE end_ms>?1",params![now]).map_err(storage)?;
 if p.minutes>0{tx.execute("INSERT INTO private_windows VALUES(?1,?2,?3)",params![uuid::Uuid::new_v4().to_string(),now,until]).map_err(storage)?;}
 tx.execute("INSERT OR REPLACE INTO settings(key,value) VALUES('private_until_ms',?1)",params![until.to_string()]).map_err(storage)?;advance_epoch(&tx).map_err(storage)?;tx.commit().map_err(storage)?;
 Ok(Json(private_info(&db,now)))
}
pub fn prune(db:&Connection,cutoff:i64,now:i64)->rusqlite::Result<()>{
 db.execute("DELETE FROM local_inbox WHERE updated_ms<?1",params![cutoff])?;
 db.execute("DELETE FROM video_visits WHERE occurred_ms<?1",params![cutoff])?;
 db.execute("DELETE FROM classification_requests WHERE started_ms<?1",params![cutoff])?;
 db.execute("DELETE FROM classification_memory WHERE last_used_ms<?1 AND subject NOT IN(SELECT subject FROM category_overrides)",params![cutoff])?;
 db.execute("DELETE FROM classifications WHERE updated_ms<?1 AND subject NOT IN(SELECT subject FROM classification_memory) AND subject NOT IN(SELECT subject FROM category_overrides)",params![cutoff])?;
 db.execute("DELETE FROM classification_job_state WHERE subject NOT IN(SELECT subject FROM classifications)",[])?;
 db.execute("DELETE FROM classification_attempts WHERE subject NOT IN(SELECT subject FROM classifications)",[])?;
 db.execute("DELETE FROM purged_subjects WHERE before_ms<?1",params![now-2*86400000])?;
 db.execute("DELETE FROM private_windows WHERE end_ms<?1",params![now-2*86400000])?;Ok(())
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct PresentationInput{enabled:bool}
pub async fn presentation(State(s):State<Arc<AppState>>,Json(p):Json<PresentationInput>)->ApiResult{
 s.db.lock().unwrap().execute("INSERT OR REPLACE INTO settings(key,value) VALUES('presentation_privacy',?1)",params![p.enabled.to_string()]).map_err(storage)?;
 Ok(Json(json!({"ok":true,"tracking_unchanged":true,"notification_details_hidden":p.enabled})))
}
fn storage(_:rusqlite::Error)->(StatusCode,String){err(StatusCode::INTERNAL_SERVER_ERROR,"privacy_storage_failed")}
#[cfg(test)]mod tests{
 use super::*;
 fn db()->Connection{let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();db}
 #[test]fn full_wipe_clears_ai_memory_and_stale_delivery_without_removing_owner_rules(){
  let db=db();db.execute("INSERT INTO category_overrides VALUES('app:editor.exe','Development',1)",[]).unwrap();
  crate::classification_store::remember(&db,"app:editor.exe","Development",0.9,"owner context","openai","test",10,false).unwrap();let old=features::read_setting(&db,"activity_data_epoch");clear_activity(&db,100).unwrap();
  assert!(!classification_store::restore_cached(&db,"app:editor.exe",101).unwrap());assert_ne!(features::read_setting(&db,"activity_data_epoch"),old);
  assert_eq!(db.query_row("SELECT COUNT(*) FROM category_overrides",[],|r|r.get::<_,i64>(0)).unwrap(),1);assert_eq!(rejection(&db,99,None).unwrap(),Some("ignored_deleted"));assert_eq!(rejection(&db,101,None).unwrap(),None);
 }
 #[test]fn scoped_erasure_and_private_window_reject_replayed_events_after_resuming(){
  let db=db();db.execute_batch("INSERT INTO sessions VALUES('a',1,50,'brave.exe',0,'www.youtube.com',NULL);INSERT INTO sessions VALUES('b',1,50,'editor.exe',0,NULL,NULL);
   INSERT INTO private_windows VALUES('private',200,300);").unwrap();
  let tx=db.unchecked_transaction().unwrap();erase_subjects(&tx,&["domain:youtube.com".into()],100).unwrap();tx.commit().unwrap();
  assert_eq!(db.query_row("SELECT COUNT(*) FROM sessions",[],|r|r.get::<_,i64>(0)).unwrap(),1);
  assert_eq!(rejection(&db,50,Some("domain:www.youtube.com")).unwrap(),Some("ignored_deleted"));assert_eq!(rejection(&db,50,Some("domain:youtube.com.evil.com")).unwrap(),None);
  assert_eq!(rejection(&db,250,None).unwrap(),Some("ignored_private"));assert_eq!(rejection(&db,301,None).unwrap(),None);
 }
}

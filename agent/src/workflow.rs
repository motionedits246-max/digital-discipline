//! Local-first user-created focus profiles and task outcomes (N01).
//! These records are NOT observed time and never create enforcement rules on their own.
use crate::{AppState,ApiResult,err,now_ms};
use crate::domain;
use axum::{extract::{Path,State},http::StatusCode,Json};
use rusqlite::{params,Connection,OptionalExtension};
use serde::Deserialize;
use serde_json::{json,Value};
use std::sync::Arc;
use uuid::Uuid;

pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 let version:i32=db.query_row("PRAGMA user_version",[],|r|r.get(0))?;
 if version<8{
   db.execute_batch("BEGIN;
    CREATE TABLE IF NOT EXISTS workflow_profiles (
      id TEXT PRIMARY KEY,name TEXT NOT NULL,apps_json TEXT NOT NULL,domains_json TEXT NOT NULL,
      created_ms INTEGER NOT NULL,updated_ms INTEGER NOT NULL,archived INTEGER NOT NULL DEFAULT 0);
    CREATE TABLE IF NOT EXISTS workflow_tasks (
      id TEXT PRIMARY KEY,profile_id TEXT REFERENCES workflow_profiles(id) ON DELETE SET NULL,
      title TEXT NOT NULL,outcome TEXT NOT NULL DEFAULT '',state TEXT NOT NULL DEFAULT 'draft',
      created_ms INTEGER NOT NULL,updated_ms INTEGER NOT NULL,finished_ms INTEGER);
    CREATE INDEX IF NOT EXISTS idx_workflow_tasks_profile ON workflow_tasks(profile_id,updated_ms);
    ALTER TABLE focus_sessions ADD COLUMN task_id TEXT;
    PRAGMA user_version=8;COMMIT;")?;
 }
 Ok(())
}
fn valid_name(name:&str,max:usize)->bool{
 !name.trim().is_empty() && name.chars().count()<=max && !name.chars().any(|c|c.is_control())
}
fn app_name(s:&str)->bool{
 !s.is_empty()&&s.len()<=100&&s.bytes().all(|b|b.is_ascii_alphanumeric()||b"_.-".contains(&b))
}
fn valid_status(s:&str)->bool{matches!(s,"draft"|"active"|"paused"|"completed"|"abandoned")}
fn allowed_transition(old:&str,new:&str)->bool{
 match old{
  "draft"=>["active","abandoned"].contains(&new),
  "active"=>["paused","completed","abandoned"].contains(&new),
  "paused"=>["active","completed","abandoned"].contains(&new),
  "completed"|"abandoned"=>false,_=>false
 }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewProfile{name:String,apps:Vec<String>,domains:Vec<String>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NewTask{title:String,profile_id:Option<String>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateTask{state:String,#[serde(default)] outcome:Option<String>}
pub async fn list(State(s):State<Arc<AppState>>)->ApiResult{
 let db=s.db.lock().unwrap();
 let mut q=db.prepare("SELECT id,name,apps_json,domains_json,created_ms,updated_ms FROM workflow_profiles WHERE archived=0 ORDER BY updated_ms DESC LIMIT 200")
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 let profiles=q.query_map([],|r|{
  let apps:String=r.get(2)?;let domains:String=r.get(3)?;
  Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"apps":serde_json::from_str::<Value>(&apps).unwrap_or(json!([])),
  "domains":serde_json::from_str::<Value>(&domains).unwrap_or(json!([])),
  "created_ms":r.get::<_,i64>(4)?,"updated_ms":r.get::<_,i64>(5)?}))
 }).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?
  .collect::<rusqlite::Result<Vec<_>>>().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 let mut t=db.prepare("SELECT id,profile_id,title,outcome,state,created_ms,updated_ms,finished_ms FROM workflow_tasks ORDER BY updated_ms DESC LIMIT 500")
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 let tasks=t.query_map([],|r|Ok(json!({"id":r.get::<_,String>(0)?,"profile_id":r.get::<_,Option<String>>(1)?,
  "title":r.get::<_,String>(2)?,"outcome":r.get::<_,String>(3)?,"state":r.get::<_,String>(4)?,
  "created_ms":r.get::<_,i64>(5)?,"updated_ms":r.get::<_,i64>(6)?,"finished_ms":r.get::<_,Option<i64>>(7)?})))
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?.collect::<rusqlite::Result<Vec<_>>>()
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 Ok(Json(json!({"ok":true,"profiles":profiles,"tasks":tasks,"task_outcomes":"self_reported_not_measured",
 "time_is_observed_separately":true,"may_be_truncated":profiles.len()==200||tasks.len()==500})))
}
pub async fn add_profile(State(s):State<Arc<AppState>>,Json(p):Json<NewProfile>)->ApiResult{
 if !valid_name(&p.name,80)||p.apps.len()>40||p.domains.len()>40||
  p.apps.iter().any(|a|!app_name(a))||p.domains.iter().any(|d|!domain::valid_domain(d)){
   return Err(err(StatusCode::BAD_REQUEST,"invalid_profile"));
 }
 let id=Uuid::new_v4().to_string();let now=now_ms();
 s.db.lock().unwrap().execute("INSERT INTO workflow_profiles(id,name,apps_json,domains_json,created_ms,updated_ms) VALUES(?1,?2,?3,?4,?5,?5)",
  params![id,p.name,serde_json::to_string(&p.apps).unwrap(),serde_json::to_string(&p.domains).unwrap(),now])
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 Ok(Json(json!({"ok":true,"id":id})))
}
pub async fn archive_profile(Path(id):Path<String>,State(s):State<Arc<AppState>>)->ApiResult{
 if Uuid::parse_str(&id).is_err(){return Err(err(StatusCode::BAD_REQUEST,"invalid_id"));}
 let count=s.db.lock().unwrap().execute("UPDATE workflow_profiles SET archived=1,updated_ms=?2 WHERE id=?1 AND archived=0",params![id,now_ms()])
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 Ok(Json(json!({"ok":true,"archived":count>0})))
}
pub async fn add_task(State(s):State<Arc<AppState>>,Json(p):Json<NewTask>)->ApiResult{
 if !valid_name(&p.title,160){return Err(err(StatusCode::BAD_REQUEST,"invalid_task_title"));}
 let db=s.db.lock().unwrap();
 if let Some(profile)=&p.profile_id {
  if Uuid::parse_str(profile).is_err(){return Err(err(StatusCode::BAD_REQUEST,"invalid_profile_id"));}
  let found=db.query_row("SELECT 1 FROM workflow_profiles WHERE id=?1 AND archived=0",params![profile],|r|r.get::<_,i64>(0)).optional()
   .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
  if found.is_none(){return Err(err(StatusCode::BAD_REQUEST,"profile_not_found"));}
 }
 let id=Uuid::new_v4().to_string();let now=now_ms();
 db.execute("INSERT INTO workflow_tasks(id,profile_id,title,state,created_ms,updated_ms) VALUES(?1,?2,?3,'draft',?4,?4)",
  params![id,p.profile_id,p.title,now]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 Ok(Json(json!({"ok":true,"id":id,"state":"draft"})))
}
pub async fn transition_task(Path(id):Path<String>,State(s):State<Arc<AppState>>,Json(p):Json<UpdateTask>)->ApiResult{
 if Uuid::parse_str(&id).is_err()||!valid_status(&p.state)||p.outcome.as_ref().is_some_and(|x|x.chars().count()>1000||x.chars().any(|c|c.is_control())){
  return Err(err(StatusCode::BAD_REQUEST,"invalid_task_update"));
 }
 let mut db=s.db.lock().unwrap();let tx=db.transaction().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 let old=tx.query_row("SELECT state FROM workflow_tasks WHERE id=?1",params![id],|r|r.get::<_,String>(0)).optional()
  .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?
  .ok_or_else(||err(StatusCode::NOT_FOUND,"task_not_found"))?;
 if !allowed_transition(&old,&p.state){return Err(err(StatusCode::CONFLICT,"invalid_task_transition"));}
 let finished=if matches!(p.state.as_str(),"completed"|"abandoned"){Some(now_ms())}else{None};
 tx.execute("UPDATE workflow_tasks SET state=?2,outcome=COALESCE(?3,outcome),updated_ms=?4,finished_ms=?5 WHERE id=?1",
  params![id,p.state,p.outcome,now_ms(),finished]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 tx.commit().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"workflow_storage_failed"))?;
 Ok(Json(json!({"ok":true,"id":id,"state":p.state,"reported_outcome":true})))
}
#[cfg(test)]
mod tests{
 use super::*;
 #[test] fn task_state_requires_explicit_transition(){
  assert!(valid_status("completed"));assert!(!valid_status("verified"));
  assert!(!app_name("C:\\Users\\private.exe"));
  assert!(app_name("cursor.exe"));
  assert!(allowed_transition("draft","active"));
  assert!(allowed_transition("active","paused"));
  assert!(allowed_transition("paused","completed"));
  assert!(!allowed_transition("completed","active"));
  assert!(!allowed_transition("draft","completed"));
 }
 #[test] fn migration_is_safe_on_old_schema(){
  let db=Connection::open_in_memory().unwrap();
  db.execute_batch("CREATE TABLE focus_sessions(id TEXT PRIMARY KEY); PRAGMA user_version=7;").unwrap();
  migrate(&db).unwrap();
  assert!(db.prepare("SELECT task_id FROM focus_sessions").is_ok());
  migrate(&db).unwrap();
 }
}

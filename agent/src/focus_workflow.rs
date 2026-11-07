//! Persisted focus context, work/break transitions and owner-written resume notes.
use crate::{AppState,ApiResult,err,now_ms,policy};
use axum::{extract::{State,Path},http::StatusCode,Json};
use rusqlite::{params,Connection,OptionalExtension};use serde::Deserialize;use serde_json::{json,Value};use std::sync::Arc;
pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 let columns={let mut q=db.prepare("PRAGMA table_info(focus_sessions)")?;let names=q.query_map([],|r|r.get::<_,String>(1))?.collect::<rusqlite::Result<Vec<_>>>()?;names};
 for (name,definition) in [("profile_id","profile_id TEXT"),("context_json","context_json TEXT NOT NULL DEFAULT '{}'"),("planned_ms","planned_ms INTEGER NOT NULL DEFAULT 0")]{if !columns.iter().any(|n|n==name){db.execute_batch(&format!("ALTER TABLE focus_sessions ADD COLUMN {definition};"))?;}}
 db.execute_batch("CREATE TABLE IF NOT EXISTS focus_cycles(focus_id TEXT PRIMARY KEY REFERENCES focus_sessions(id) ON DELETE CASCADE,
  state TEXT NOT NULL,phase TEXT NOT NULL,stage INTEGER NOT NULL,ends_ms INTEGER NOT NULL,remaining_ms INTEGER NOT NULL,work_ms INTEGER NOT NULL,break_ms INTEGER NOT NULL,cycles INTEGER NOT NULL,break_policy TEXT NOT NULL,version INTEGER NOT NULL DEFAULT 0);
 CREATE TABLE IF NOT EXISTS focus_cycle_transitions(id TEXT PRIMARY KEY,focus_id TEXT NOT NULL REFERENCES focus_sessions(id) ON DELETE CASCADE,sequence INTEGER NOT NULL,from_state TEXT NOT NULL,to_state TEXT NOT NULL,occurred_ms INTEGER NOT NULL,reason TEXT NOT NULL,UNIQUE(focus_id,sequence));
 CREATE TABLE IF NOT EXISTS focus_reviews(focus_id TEXT PRIMARY KEY REFERENCES focus_sessions(id) ON DELETE CASCADE,outcome TEXT NOT NULL DEFAULT '',next_step TEXT NOT NULL DEFAULT '',dismissed INTEGER NOT NULL DEFAULT 0,updated_ms INTEGER NOT NULL);
 ")
}
#[derive(Clone,Deserialize)]#[serde(deny_unknown_fields)]pub struct CycleInput{pub work_minutes:u32,pub break_minutes:u32,pub cycles:u32,pub break_policy:String}
impl CycleInput{pub fn valid(&self)->bool{(1..=120).contains(&self.work_minutes)&&(1..=60).contains(&self.break_minutes)&&(1..=8).contains(&self.cycles)&&["continue","soften"].contains(&self.break_policy.as_str())}pub fn planned_ms(&self)->i64{(self.work_minutes as i64*self.cycles as i64+self.break_minutes as i64*(self.cycles as i64-1))*60000}}
pub fn context(db:&Connection,task:Option<&str>,profile:Option<&str>)->Result<(Option<String>,Value),(StatusCode,String)>{
 let mut profile=profile.map(str::to_owned);let mut intended=None;
 if let Some(task)=task{
  let row=db.query_row("SELECT profile_id,title,state FROM workflow_tasks WHERE id=?1",params![task],|r|Ok((r.get::<_,Option<String>>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional().map_err(storage)?.ok_or_else(||err(StatusCode::BAD_REQUEST,"task_not_found"))?;
  if ["completed","abandoned"].contains(&row.2.as_str()){return Err(err(StatusCode::CONFLICT,"task_already_finished"));}
  if profile.is_some()&&row.0.is_some()&&profile!=row.0{return Err(err(StatusCode::BAD_REQUEST,"task_profile_mismatch"));}
  if profile.is_none(){profile=row.0;}intended=Some(row.1);
 }
 let mut result=json!({"intended_outcome":intended,"apps":[],"domains":[]});
 if let Some(id)=&profile{
  let row=db.query_row("SELECT name,apps_json,domains_json FROM workflow_profiles WHERE id=?1 AND archived=0",params![id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,String>(2)?))).optional().map_err(storage)?.ok_or_else(||err(StatusCode::BAD_REQUEST,"profile_not_found"))?;
  result["profile_name"]=json!(row.0);result["apps"]=serde_json::from_str(&row.1).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"invalid_saved_profile"))?;result["domains"]=serde_json::from_str(&row.2).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"invalid_saved_profile"))?;
 }
 Ok((profile,result))
}
pub fn attach(db:&Connection,id:&str,task:Option<&str>,profile:Option<&str>,context:&Value,cycle:Option<&CycleInput>,now:i64,planned:i64)->rusqlite::Result<()>{
 db.execute("UPDATE focus_sessions SET task_id=?2,profile_id=?3,context_json=?4,planned_ms=?5 WHERE id=?1",params![id,task,profile,context.to_string(),planned])?;
 if let Some(task)=task{db.execute("UPDATE workflow_tasks SET state='active',updated_ms=?2 WHERE id=?1 AND state IN('draft','paused')",params![task,now])?;}
 if let Some(c)=cycle{db.execute("INSERT INTO focus_cycles(focus_id,state,phase,stage,ends_ms,remaining_ms,work_ms,break_ms,cycles,break_policy) VALUES(?1,'work','work',0,?2,0,?3,?4,?5,?6)",params![id,now+c.work_minutes as i64*60000,c.work_minutes as i64*60000,c.break_minutes as i64*60000,c.cycles,c.break_policy])?;}
 Ok(())
}
#[derive(Clone)]struct Cycle{id:String,state:String,phase:String,stage:i64,end:i64,remaining:i64,work:i64,rest:i64,cycles:i64,policy:String,version:i64}
fn cycle(db:&Connection,id:&str)->rusqlite::Result<Option<Cycle>>{db.query_row("SELECT focus_id,state,phase,stage,ends_ms,remaining_ms,work_ms,break_ms,cycles,break_policy,version FROM focus_cycles WHERE focus_id=?1",params![id],|r|Ok(Cycle{id:r.get(0)?,state:r.get(1)?,phase:r.get(2)?,stage:r.get(3)?,end:r.get(4)?,remaining:r.get(5)?,work:r.get(6)?,rest:r.get(7)?,cycles:r.get(8)?,policy:r.get(9)?,version:r.get(10)?})).optional()}
fn persist(db:&Connection,c:&Cycle,old:&str,at:i64,reason:&str)->rusqlite::Result<()>{
 db.execute("UPDATE focus_cycles SET state=?2,phase=?3,stage=?4,ends_ms=?5,remaining_ms=?6,version=?7 WHERE focus_id=?1",params![c.id,c.state,c.phase,c.stage,c.end,c.remaining,c.version])?;
 db.execute("INSERT INTO focus_cycle_transitions VALUES(?1,?2,?3,?4,?5,?6,?7)",params![uuid::Uuid::new_v4().to_string(),c.id,c.version,old,c.state,at,reason])?;
 if c.state=="finished"{db.execute("UPDATE focus_sessions SET ended_ms=COALESCE(ended_ms,?2) WHERE id=?1",params![c.id,at])?;}Ok(())
}
fn next(c:&mut Cycle,at:i64){
 if c.phase=="work"{if c.stage+1>=c.cycles{c.state="finished".into();c.end=at;}else{c.state="break".into();c.phase="break".into();c.end=at+c.rest;}}
 else{c.stage+=1;c.state="work".into();c.phase="work".into();c.end=at+c.work;}c.remaining=0;c.version+=1;
}
pub fn advance(db:&Connection,now:i64)->rusqlite::Result<()>{
 if db.is_autocommit(){let tx=db.unchecked_transaction()?;advance_inner(&tx,now)?;tx.commit()}else{advance_inner(db,now)}
}
fn advance_inner(db:&Connection,now:i64)->rusqlite::Result<()>{
 let ids={let mut q=db.prepare("SELECT focus_id FROM focus_cycles WHERE state IN('work','break') AND ends_ms<=?1 LIMIT 20")?;let ids=q.query_map(params![now],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;ids};
 for id in ids{let Some(mut c)=cycle(db,&id)?else{continue;};for _ in 0..16{if c.state=="finished"||c.end>now{break;}let old=c.state.clone();let boundary=c.end;next(&mut c,boundary);persist(db,&c,&old,boundary,"scheduled_transition")?;}}
 Ok(())
}
pub fn finish_all(db:&Connection,now:i64)->rusqlite::Result<()>{
 let ids={let mut q=db.prepare("SELECT focus_id FROM focus_cycles WHERE state!='finished'")?;let rows=q.query_map([],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;rows};
 for id in ids{if let Some(mut c)=cycle(db,&id)?{let old=c.state.clone();c.state="finished".into();c.version+=1;c.end=now;persist(db,&c,&old,now,"owner_finish")?;}}Ok(())
}
pub fn augment(db:&Connection,focus:&mut Value)->rusqlite::Result<()>{
 let id=focus["id"].as_str().unwrap_or("");let(row_profile,task,context):(Option<String>,Option<String>,String)=db.query_row("SELECT profile_id,task_id,context_json FROM focus_sessions WHERE id=?1",params![id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?)))?;
 let c=cycle(db,id)?;focus["profile_id"]=json!(row_profile);focus["task_id"]=json!(task);focus["context"]=serde_json::from_str(&context).unwrap_or(json!({}));
 if let Some(c)=c{focus["cycle"]=json!({"state":c.state,"phase":c.phase,"stage":c.stage,"cycles":c.cycles,"ends_ms":c.end,"remaining_ms":c.remaining,"break_policy":c.policy,"version":c.version});}
 Ok(())
}
pub fn restrictions(db:&Connection,id:&str,difficulty:&str)->rusqlite::Result<Option<String>>{
 Ok(match cycle(db,id)?{Some(c)if c.state=="finished"=>None,Some(c)if c.state=="paused"=>None,Some(c)if c.state=="break"&&c.policy=="soften"=>Some("Balanced".into()),_=>Some(difficulty.into())})
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Action{action:String,version:i64,minutes:Option<u32>}
pub fn apply_action(db:&Connection,id:&str,p:&Action,now:i64)->ApiResult{
 let tx=db.unchecked_transaction().map_err(storage)?;advance(&tx,now).map_err(storage)?;let mut c=cycle(&tx,id).map_err(storage)?.ok_or_else(||err(StatusCode::NOT_FOUND,"cycle_not_found"))?;
 if c.version!=p.version{return Err(err(StatusCode::CONFLICT,"cycle_changed_refresh"));}if c.state=="finished"{return Err(err(StatusCode::CONFLICT,"cycle_finished"));}
 let old=c.state.clone();match p.action.as_str(){
  "pause" if c.state!="paused"=>{c.remaining=(c.end-now).max(0);c.state="paused".into();c.version+=1;},
  "resume" if c.state=="paused"=>{c.state=c.phase.clone();c.end=now+c.remaining;c.remaining=0;c.version+=1;},
  "skip"=>{next(&mut c,now);},
  "extend"=>{let minutes=p.minutes.filter(|m|(1..=120).contains(m)).ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_extension"))?;if c.state=="paused"{c.remaining+=minutes as i64*60000;}else{c.end+=minutes as i64*60000;}c.version+=1;},
  "finish"=>{c.state="finished".into();c.end=now;c.version+=1;},_=>return Err(err(StatusCode::BAD_REQUEST,"invalid_cycle_action"))
 }
 persist(&tx,&c,&old,now,&p.action).map_err(storage)?;
 if c.state!="finished"{
  let pending=if c.state=="paused"{c.remaining}else{(c.end-now).max(0)};let future_work=(c.cycles-c.stage-1)*c.work;let future_breaks=if c.phase=="work"{c.cycles-c.stage-1}else{(c.cycles-c.stage-2).max(0)}*c.rest;
  tx.execute("UPDATE focus_sessions SET end_ms=?2 WHERE id=?1",params![id,now+pending+future_work+future_breaks]).map_err(storage)?;
 }
 tx.commit().map_err(storage)?;Ok(Json(json!({"ok":true,"state":c.state,"version":c.version,"observed_time_unchanged":true})))
}
pub async fn action(Path(id):Path<String>,State(s):State<Arc<AppState>>,Json(p):Json<Action>)->ApiResult{if uuid::Uuid::parse_str(&id).is_err(){return Err(err(StatusCode::BAD_REQUEST,"invalid_id"));}apply_action(&s.db.lock().unwrap(),&id,&p,now_ms())}
pub async fn reviews(State(s):State<Arc<AppState>>)->ApiResult{
 let db=s.db.lock().unwrap();advance(&db,now_ms()).map_err(storage)?;
 let mut q=db.prepare("SELECT f.id,f.start_ms,min(f.end_ms,COALESCE(f.ended_ms,f.end_ms)),f.planned_ms,f.context_json,COALESCE(r.outcome,''),COALESCE(r.next_step,''),COALESCE(r.dismissed,0),t.state,t.outcome FROM focus_sessions f LEFT JOIN focus_reviews r ON r.focus_id=f.id LEFT JOIN workflow_tasks t ON t.id=f.task_id WHERE f.ended_ms IS NOT NULL OR (f.end_ms<=?1 AND NOT EXISTS(SELECT 1 FROM focus_cycles c WHERE c.focus_id=f.id AND c.state='paused')) ORDER BY f.start_ms DESC LIMIT 50").map_err(storage)?;
 let mut out=Vec::new();let mut rows=q.query(params![now_ms()]).map_err(storage)?;
 while let Some(row)=rows.next().map_err(storage)?{
  let start:i64=row.get(1).map_err(storage)?;let end:i64=row.get(2).map_err(storage)?;let raw:String=row.get(4).map_err(storage)?;let context:Value=serde_json::from_str(&raw).unwrap_or(json!({}));
  let apps=serde_json::from_value::<Vec<String>>(context["apps"].clone()).unwrap_or_default();let domains=serde_json::from_value::<Vec<String>>(context["domains"].clone()).unwrap_or_default();
  let sessions=crate::get_sessions(&db,start,end,50000).map_err(storage)?;let measured=policy::union_ms(sessions.iter().filter(|s|!s.idle).map(|s|(s.start_ms,s.end_ms)).collect(),start,end);
  let eligible=if apps.is_empty()&&domains.is_empty(){None}else{Some(crate::analytics::active_target_ms(&db,&domains,&apps,&[],&[],&[],start,end).map_err(storage)?)};
  let switches=crate::insights::count_switches(&sessions.iter().map(|s|(s.app.clone(),s.start_ms,s.end_ms,s.idle)).collect::<Vec<_>>(),start,end);
  out.push(json!({"id":row.get::<_,String>(0).map_err(storage)?,"start_ms":start,"end_ms":end,"planned_ms":row.get::<_,i64>(3).map_err(storage)?,"context":context,"observed_active_ms":measured,"observed_eligible_ms":eligible,"observed_switches":switches,"owner_outcome":row.get::<_,String>(5).map_err(storage)?,"next_step":row.get::<_,String>(6).map_err(storage)?,"dismissed":row.get::<_,i64>(7).map_err(storage)?!=0,"task_state":row.get::<_,Option<String>>(8).map_err(storage)?,"task_outcome":row.get::<_,Option<String>>(9).map_err(storage)?,"coverage":if sessions.is_empty(){"no_observations"}else if sessions.len()==50000{"limited"}else{"observed_intervals_only"}}));
 }
 Ok(Json(json!({"ok":true,"reviews":out,"notes_local_only":true})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct ReviewInput{outcome:String,next_step:String,#[serde(default)]dismissed:bool}
pub async fn save_review(Path(id):Path<String>,State(s):State<Arc<AppState>>,Json(p):Json<ReviewInput>)->ApiResult{
 if uuid::Uuid::parse_str(&id).is_err()||p.outcome.chars().count()>1000||p.next_step.chars().count()>1000||p.outcome.chars().chain(p.next_step.chars()).any(|c|c.is_control()&&c!='\n'){return Err(err(StatusCode::BAD_REQUEST,"invalid_review"));}
 let db=s.db.lock().unwrap();if !db.query_row("SELECT EXISTS(SELECT 1 FROM focus_sessions WHERE id=?1)",params![id],|r|r.get::<_,bool>(0)).map_err(storage)?{return Err(err(StatusCode::NOT_FOUND,"focus_not_found"));}
 db.execute("INSERT INTO focus_reviews VALUES(?1,?2,?3,?4,?5) ON CONFLICT(focus_id) DO UPDATE SET outcome=excluded.outcome,next_step=excluded.next_step,dismissed=excluded.dismissed,updated_ms=excluded.updated_ms",params![id,p.outcome,p.next_step,p.dismissed as u8,now_ms()]).map_err(storage)?;
 Ok(Json(json!({"ok":true,"self_reported":true})))
}
fn storage(_:rusqlite::Error)->(StatusCode,String){err(StatusCode::INTERNAL_SERVER_ERROR,"focus_workflow_storage_failed")}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn cycles_survive_reload_and_sleep_without_creating_observed_work(){
  let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();let id=uuid::Uuid::new_v4().to_string();db.execute("INSERT INTO focus_sessions(id,start_ms,end_ms,difficulty,domains_json) VALUES(?1,1000,181000,'Strict','[]')",params![id]).unwrap();let c=CycleInput{work_minutes:1,break_minutes:1,cycles:2,break_policy:"soften".into()};attach(&db,&id,None,None,&json!({}),Some(&c),1000,180000).unwrap();advance(&db,62000).unwrap();let saved=cycle(&db,&id).unwrap().unwrap();assert_eq!(saved.state,"break");assert_eq!(restrictions(&db,&id,"Strict").unwrap(),Some("Balanced".into()));
  let _=apply_action(&db,&id,&Action{action:"pause".into(),version:saved.version,minutes:None},63000).unwrap();advance(&db,500000).unwrap();assert_eq!(cycle(&db,&id).unwrap().unwrap().state,"paused");
  assert_eq!(db.query_row("SELECT COUNT(*) FROM sessions",[],|r|r.get::<_,i64>(0)).unwrap(),0);
  let version=cycle(&db,&id).unwrap().unwrap().version;let _=apply_action(&db,&id,&Action{action:"resume".into(),version,minutes:None},500000).unwrap();assert!(apply_action(&db,&id,&Action{action:"extend".into(),version,minutes:Some(5)},500000).is_err());
  advance(&db,999999).unwrap();assert_eq!(cycle(&db,&id).unwrap().unwrap().state,"finished");
 }
}

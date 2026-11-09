//! Grouped local status records. Inbox entries do not imply toast delivery.
use crate::{AppState,ApiResult,err,now_ms};use axum::{extract::{State,Path},http::StatusCode,Json};
use rusqlite::{Connection,params};use serde::Deserialize;use serde_json::{json,Value};use std::sync::Arc;
pub fn migrate(db:&Connection)->rusqlite::Result<()>{
 db.execute_batch("CREATE TABLE IF NOT EXISTS local_inbox(id TEXT PRIMARY KEY,topic TEXT NOT NULL UNIQUE,kind TEXT NOT NULL,title TEXT NOT NULL,detail TEXT NOT NULL,action TEXT NOT NULL,subject TEXT,created_ms INTEGER NOT NULL,updated_ms INTEGER NOT NULL,occurrences INTEGER NOT NULL DEFAULT 1,dismissed_ms INTEGER);
 CREATE INDEX IF NOT EXISTS idx_inbox_current ON local_inbox(dismissed_ms,updated_ms);")?;
 db.execute("INSERT OR IGNORE INTO settings(key,value) VALUES('inbox_started_ms',?1)",params![now_ms().to_string()])?;Ok(())
}
pub fn record(db:&Connection,topic:&str,kind:&str,title:&str,detail:Value,action:&str,subject:Option<&str>,now:i64)->rusqlite::Result<()>{
 db.execute("INSERT INTO local_inbox(id,topic,kind,title,detail,action,subject,created_ms,updated_ms) VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?8)
 ON CONFLICT(topic) DO UPDATE SET detail=excluded.detail,updated_ms=excluded.updated_ms,occurrences=occurrences+1",params![uuid::Uuid::new_v4().to_string(),topic,kind,title,detail.to_string(),action,subject,now])?;Ok(())
}
pub fn completed(db:&Connection,now:i64)->rusqlite::Result<()>{
 let since=crate::features::read_setting(db,"inbox_started_ms").parse::<i64>().unwrap_or(now);
 let ids={let mut q=db.prepare("SELECT id FROM focus_sessions f WHERE start_ms>=?1 AND (ended_ms IS NOT NULL OR end_ms<=?2) AND NOT EXISTS(SELECT 1 FROM focus_cycles c WHERE c.focus_id=f.id AND c.state='paused') AND NOT EXISTS(SELECT 1 FROM local_inbox i WHERE i.topic='focus:'||f.id) ORDER BY start_ms DESC LIMIT 20")?;let ids=q.query_map(params![since,now],|r|r.get::<_,String>(0))?.collect::<rusqlite::Result<Vec<_>>>()?;ids};
 for id in ids{record(db,&format!("focus:{id}"),"focus","Focus timebox ended",json!({"focus_id":id,"time_is_not_task_completion":true}),"Focus & Goals",None,now)?;}Ok(())
}
pub async fn list(State(s):State<Arc<AppState>>)->ApiResult{
 let db=s.db.lock().unwrap();completed(&db,now_ms()).map_err(storage)?;let mut q=db.prepare("SELECT id,kind,title,detail,action,created_ms,updated_ms,occurrences,dismissed_ms FROM local_inbox ORDER BY dismissed_ms IS NULL DESC,updated_ms DESC LIMIT 200").map_err(storage)?;
 let items=q.query_map([],|r|{let detail:String=r.get(3)?;Ok(json!({"id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,"title":r.get::<_,String>(2)?,"detail":serde_json::from_str::<Value>(&detail).unwrap_or(json!({})),"action":r.get::<_,String>(4)?,"created_ms":r.get::<_,i64>(5)?,"updated_ms":r.get::<_,i64>(6)?,"occurrences":r.get::<_,i64>(7)?,"dismissed_ms":r.get::<_,Option<i64>>(8)?}))}).map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
 Ok(Json(json!({"ok":true,"items":items,"delivery":"local_inbox_and_desktop_overlay","lock_screen_text":"Generic status titles; no app names, URLs or notes","may_be_truncated":items.len()==200})))
}
#[derive(Deserialize)]#[serde(deny_unknown_fields)]pub struct Dismiss{dismissed:bool}
pub async fn dismiss(Path(id):Path<String>,State(s):State<Arc<AppState>>,Json(p):Json<Dismiss>)->ApiResult{if uuid::Uuid::parse_str(&id).is_err(){return Err(err(StatusCode::BAD_REQUEST,"invalid_id"));}s.db.lock().unwrap().execute("UPDATE local_inbox SET dismissed_ms=?2 WHERE id=?1",params![id,if p.dismissed{Some(now_ms())}else{None}]).map_err(storage)?;Ok(Json(json!({"ok":true})))}
fn storage(_:rusqlite::Error)->(StatusCode,String){err(StatusCode::INTERNAL_SERVER_ERROR,"inbox_storage_failed")}
pub fn safe_native_app(app:&str)->bool{
 !app.is_empty()&&app.len()<=100&&app.ends_with(".exe")&&app.bytes().all(|b|b.is_ascii_alphanumeric()||b"-_.".contains(&b))&&!["explorer.exe","taskmgr.exe","digital-discipline-desktop.exe","digital-discipline-agent.exe","narrator.exe","nvda.exe","jfw.exe","utilman.exe","osk.exe","logonui.exe","lockapp.exe","msmpeng.exe","securityhealthservice.exe","securityhealthsystray.exe","cmd.exe","powershell.exe","pwsh.exe","windowsterminal.exe"].contains(&app)
}
pub fn native_hint(db:&Connection,focus:&Value,current:Option<&crate::SessionDto>,now:i64)->Option<Value>{
 if crate::data_lifecycle::is_private(db,now)||focus["cycle"]["state"]=="paused"||focus["cycle"]["state"]=="break"{return None;}
 let session=current?;if session.idle||now-session.start_ms<10000{return None;}
 if !focus["context"]["native_apps"].as_array().is_some_and(|apps|apps.iter().any(|a|a.as_str()==Some(session.app.as_str()))){return None;}
 let id=focus["id"].as_str()?;let topic=format!("native:{id}:{}",session.app);
 let seen=db.query_row("SELECT EXISTS(SELECT 1 FROM local_inbox WHERE topic=?1)",params![topic],|r|r.get::<_,bool>(0)).unwrap_or(true);
 if !seen{let _=record(db,&topic,"native_warning","Selected app warning",json!({"focus_id":id,"capability":"warning_only"}),"Focus & Goals",Some(&format!("app:{}",session.app)),now);}
 Some(json!({"kind":"native_app_warning","since_ms":session.start_ms,"text":"You selected this app for a focus warning. Return to your chosen task when ready.","capability":"warning_only_overlay_visible","not_enforced":true}))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn repeated_status_is_grouped_and_dismissal_survives_another_occurrence(){let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();record(&db,"worker:test","health","Worker needs attention",json!({"code":"retryable"}),"Settings & Privacy",None,10).unwrap();db.execute("UPDATE local_inbox SET dismissed_ms=11",[]).unwrap();record(&db,"worker:test","health","Worker needs attention",json!({"code":"retryable"}),"Settings & Privacy",None,12).unwrap();assert_eq!(db.query_row("SELECT occurrences FROM local_inbox",[],|r|r.get::<_,i64>(0)).unwrap(),2);assert_eq!(db.query_row("SELECT dismissed_ms FROM local_inbox",[],|r|r.get::<_,i64>(0)).unwrap(),11);}
 #[test]fn native_warning_only_matches_the_explicit_app_and_respects_private_mode(){
  let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();let now=now_ms();let focus=json!({"id":uuid::Uuid::new_v4().to_string(),"goal_id":null,"context":{"native_apps":["my-game.exe"]}});
  let mut session=crate::SessionDto{id:"sample".into(),start_ms:now-15000,end_ms:now,app:"my-game.exe".into(),idle:false,domain:None,profile_id:None,category:"Gaming".into()};
  assert_eq!(native_hint(&db,&focus,Some(&session),now).unwrap()["capability"],"warning_only_overlay_visible");session.app="editor.exe".into();assert!(native_hint(&db,&focus,Some(&session),now).is_none());session.app="my-game.exe".into();
  db.execute("UPDATE settings SET value=?1 WHERE key='private_until_ms'",params![(now+60000).to_string()]).unwrap();assert!(native_hint(&db,&focus,Some(&session),now).is_none());
 }
 #[test]fn recovery_and_assistive_apps_cannot_be_selected_for_native_warning(){for app in ["explorer.exe","taskmgr.exe","nvda.exe","powershell.exe","../game.exe","Game.EXE"]{assert!(!safe_native_app(app));}assert!(safe_native_app("my-game.exe"));}
}

//! Consistent, complete local snapshots streamed from a separate WAL reader.
//! Provider keys are external encrypted files and are deliberately excluded.
use crate::{AppState,ApiResult,err,now_ms};
use axum::{extract::State,http::StatusCode,Json};
use rusqlite::{Connection,types::ValueRef};
use serde_json::{json,Value,Map};
use std::{fs::{self,OpenOptions},io::{Write,BufWriter},path::{Path,PathBuf},sync::{Arc,atomic::{AtomicBool,Ordering}}};
static EXPORTING:AtomicBool=AtomicBool::new(false);
struct ExportLease;impl Drop for ExportLease{fn drop(&mut self){EXPORTING.store(false,Ordering::SeqCst);}}
const SAFE_SETTINGS:&str="SELECT key,value FROM settings WHERE key IN ('paused','retention_days','idle_threshold_ms','excluded_apps','excluded_domains','video_history_visible','distraction_categories','prediction_opt_in','coach_share_opt_in','lite_mode','classification_auto','classification_provider','classification_model','classification_auto_review','classification_daily_limit')";
const TABLES:&[&str]=&["sessions","videos","video_visits","raw_events","category_overrides","classifications","classification_memory","goals","focus_sessions","focus_cycles","focus_cycle_transitions","focus_reviews","learning_sessions","temporary_exceptions","workflow_profiles","workflow_tasks","rules","rule_states","audits","device_sessions","linked_devices","local_inbox","settings"];
fn write_line(out:&mut impl Write,value:&Value)->Result<(),Box<dyn std::error::Error+Send+Sync>>{serde_json::to_writer(&mut *out,value)?;out.write_all(b"\n")?;Ok(())}
pub fn write_snapshot(db:&Connection,path:&Path)->Result<Value,Box<dyn std::error::Error+Send+Sync>>{
 let tx=db.unchecked_transaction()?;
 let version:i32=tx.query_row("PRAGMA user_version",[],|r|r.get(0))?;
 let file=OpenOptions::new().create_new(true).write(true).open(path)?;let mut out=BufWriter::with_capacity(65536,file);let at=now_ms();let epoch=crate::features::read_setting(&tx,"activity_data_epoch");
 write_line(&mut out,&json!({"format":"digital-discipline.local-snapshot.v1","storage_version":version,"application_version":env!("CARGO_PKG_VERSION"),"exported_at_ms":at,"credentials_included":false,"settings_scope":"Allowlisted preferences; credentials, enrollment, device identity and unknown settings omitted","automatic_restore_supported":false,"encoding":"UTF-8 JSON Lines"}))?;
 let mut counts=Map::new();let mut total=0u64;
 for table in TABLES{
  let query_text=if *table=="settings"{SAFE_SETTINGS.to_owned()}else{format!("SELECT * FROM {table}")};let mut query=tx.prepare(&query_text)?;let columns=query.column_names().iter().map(|s|s.to_string()).collect::<Vec<_>>();let mut rows=query.query([])?;let mut count=0u64;
  while let Some(row)=rows.next()?{
   let mut record=Map::new();for (index,column) in columns.iter().enumerate(){let value=match row.get_ref(index)?{ValueRef::Null=>Value::Null,ValueRef::Integer(n)=>json!(n),ValueRef::Real(n)=>json!(n),ValueRef::Text(t)=>json!(std::str::from_utf8(t)?),ValueRef::Blob(b)=>json!({"hex":hex::encode(b)})};record.insert(column.clone(),value);}
   write_line(&mut out,&json!({"table":table,"row":record}))?;count+=1;
  }
  total+=count;counts.insert((*table).to_owned(),json!(count));
 }
 write_line(&mut out,&json!({"complete":true,"truncated":false,"rows":total,"table_counts":counts}))?;
 out.flush()?;out.get_ref().sync_all()?;let bytes=out.get_ref().metadata()?.len();tx.commit()?;
 Ok(json!({"ok":true,"rows":total,"bytes":bytes,"complete":true,"truncated":false,"table_counts":counts,"exported_at_ms":at,"credentials_included":false,"data_epoch":epoch}))
}
fn cleanup(directory:&Path)->std::io::Result<()>{
 let mut files=fs::read_dir(directory)?.filter_map(Result::ok).filter(|e|e.path().extension().is_some_and(|x|x=="jsonl"||x=="partial")).collect::<Vec<_>>();files.sort_by_key(|e|e.metadata().ok().and_then(|m|m.modified().ok()));
 let excess=files.len().saturating_sub(3);for file in files.into_iter().take(excess){fs::remove_file(file.path())?;}Ok(())
}
pub async fn create(State(s):State<Arc<AppState>>)->ApiResult{
 if EXPORTING.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst).is_err(){return Err(err(StatusCode::CONFLICT,"snapshot_already_running"));}
 let lease=ExportLease;let path=s.db.lock().unwrap().path().map(PathBuf::from).ok_or_else(||err(StatusCode::SERVICE_UNAVAILABLE,"snapshot_storage_unavailable"))?;
 let directory=crate::data_dir().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"snapshot_directory_unavailable"))?.join("exports");
 let state=s.clone();
 let result=tokio::task::spawn_blocking(move||{
  let _lease=lease;
  let work=(||->Result<Value,Box<dyn std::error::Error+Send+Sync>>{
   fs::create_dir_all(&directory)?;cleanup(&directory)?;let id=uuid::Uuid::new_v4().to_string();let partial=directory.join(format!("{id}.partial"));let finished=directory.join(format!("{id}.jsonl"));
   let db=Connection::open_with_flags(path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY|rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX)?;db.busy_timeout(std::time::Duration::from_secs(3))?;
   let mut result=match write_snapshot(&db,&partial){Ok(v)=>v,Err(e)=>{let _=fs::remove_file(&partial);return Err(e);}};
   let db=state.db.lock().unwrap();if crate::features::read_setting(&db,"activity_data_epoch")!=result["data_epoch"].as_str().unwrap_or(""){let _=fs::remove_file(&partial);return Err("Snapshot invalidated by a privacy or deletion change".into());}
   fs::rename(&partial,&finished)?;result.as_object_mut().unwrap().remove("data_epoch");result["snapshot_id"]=json!(id);Ok(result)
  })();work.map_err(|_|())
 }).await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"snapshot_worker_failed"))?.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"snapshot_write_failed"))?;
 Ok(Json(result))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn complete_export_exceeds_old_fifty_thousand_row_limit_and_has_a_verified_footer(){
  let db=Connection::open_in_memory().unwrap();crate::initialize_schema(&db).unwrap();
  db.execute_batch("WITH RECURSIVE n(x) AS(VALUES(1) UNION ALL SELECT x+1 FROM n WHERE x<50005) INSERT INTO sessions(id,start_ms,end_ms,app,idle) SELECT printf('%d',x),x*2000,x*2000+1000,'editor.exe',0 FROM n;").unwrap();
  db.execute("INSERT INTO settings VALUES('unknown_provider_secret','fixture-secret-must-never-leave')",[]).unwrap();
  let file=std::env::temp_dir().join(format!("dd-export-{}.jsonl",uuid::Uuid::new_v4()));let result=write_snapshot(&db,&file).unwrap();assert_eq!(result["table_counts"]["sessions"],50005);assert_eq!(result["truncated"],false);
  let content=fs::read_to_string(&file).unwrap();assert!(!content.contains("fixture-secret-must-never-leave"));assert!(!content.contains("\"key\":\"device_id\""));let footer:Value=serde_json::from_str(content.lines().last().unwrap()).unwrap();assert_eq!(footer["complete"],true);assert_eq!(footer["table_counts"]["sessions"],50005);fs::remove_file(file).unwrap();
 }
}

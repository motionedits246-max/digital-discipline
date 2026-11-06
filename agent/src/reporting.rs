//! Streaming local-day reports. History work uses its own read-only WAL snapshot.
use crate::{AppState,ApiResult,err,features,midnight_local_ms,now_ms};
use axum::{extract::State,http::StatusCode,Json};
use chrono::{DateTime,Duration,Local,Utc};
use rusqlite::{params,Connection};
use serde_json::{json,Value};
use std::{collections::BTreeMap,sync::Arc};

#[derive(Default)]struct Day{active:i64,development:i64,entertainment:i64,observed:bool,end:i64}
pub fn heatmap(db:&Connection,now:i64)->rusqlite::Result<Value>{
 let today=DateTime::<Utc>::from_timestamp_millis(now).ok_or(rusqlite::Error::InvalidQuery)?.with_timezone(&Local).date_naive();
 let earliest=midnight_local_ms(today-Duration::days(364));
 let tx=db.unchecked_transaction()?;let classes=features::load_categories(&tx)?;
 let mut days:BTreeMap<String,Day>=BTreeMap::new();let mut overlaps=0u64;let mut count=0u64;
 {
  let mut q=tx.prepare("SELECT start_ms,end_ms,app,domain,idle FROM sessions WHERE end_ms>?1 AND start_ms<?2 ORDER BY start_ms,id")?;
  let mut rows=q.query(params![earliest,now])?;
  while let Some(row)=rows.next()?{
   let start:i64=row.get(0)?;let end:i64=row.get(1)?;let app:String=row.get(2)?;let domain:Option<String>=row.get(3)?;let idle=row.get::<_,i64>(4)?!=0;
   if end<=start{continue;}count+=1;
   let category=features::resolved_category(&classes,&app,domain.as_deref());
   let Some(dt)=DateTime::<Utc>::from_timestamp_millis(start.max(earliest))else{continue;};
   let mut day=dt.with_timezone(&Local).date_naive();let final_end=end.min(now);
   while midnight_local_ms(day)<final_end&&day<=today{
    let lo=start.max(midnight_local_ms(day));let hi=final_end.min(midnight_local_ms(day+Duration::days(1)));
    let entry=days.entry(day.to_string()).or_default();entry.observed=true;
    if !idle&&hi>lo{
     if lo<entry.end{overlaps+=1;}
     // Earlier local foreground evidence wins an ambiguous overlap; count it once.
     let delta=(hi-lo.max(entry.end)).max(0);entry.end=entry.end.max(hi);entry.active+=delta;
     if category=="Development"{entry.development+=delta;}if category=="Entertainment"{entry.entertainment+=delta;}
    }
    day+=Duration::days(1);
   }
  }
 }
 let values=(0..365).rev().map(|i|{let date=(today-Duration::days(i)).to_string();let d=days.get(&date);json!({"date":date,"active_ms":d.map_or(0,|x|x.active),"development_ms":d.map_or(0,|x|x.development),"entertainment_ms":d.map_or(0,|x|x.entertainment),"observed":d.is_some_and(|x|x.observed)})}).collect::<Vec<_>>();
 tx.commit()?;Ok(json!({"ok":true,"heatmap":values,"as_of_ms":now,"rows_examined":count,"truncated":false,"ambiguous_overlaps":overlaps,"scope":"local_foreground_only","timezone":Local::now().offset().to_string()}))
}
pub async fn history(State(s):State<Arc<AppState>>)->ApiResult{
 let path=s.db.lock().unwrap().path().map(str::to_owned).ok_or_else(||err(StatusCode::SERVICE_UNAVAILABLE,"history_snapshot_unavailable"))?;
 let out=tokio::task::spawn_blocking(move||{
  let db=Connection::open_with_flags(path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY|rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX)?;
  db.busy_timeout(std::time::Duration::from_secs(2))?;heatmap(&db,now_ms())
 }).await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"history_worker_failed"))?.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"history_read_failed"))?;
 Ok(Json(out))
}
#[cfg(test)]mod tests{
 use super::*;
 #[test]fn streaming_report_reconciles_overlap_and_marks_idle_as_observed(){let d=Connection::open_in_memory().unwrap();d.execute_batch("CREATE TABLE sessions(id TEXT PRIMARY KEY,start_ms INTEGER,end_ms INTEGER,app TEXT,domain TEXT,idle INTEGER);CREATE TABLE classifications(subject TEXT,state TEXT,category TEXT);CREATE TABLE category_overrides(subject TEXT,category TEXT);").unwrap();let at=now_ms();d.execute("INSERT INTO sessions VALUES('a',?1,?2,'code.exe',NULL,0)",params![at-10000,at-5000]).unwrap();d.execute("INSERT INTO sessions VALUES('b',?1,?2,'brave.exe',NULL,0)",params![at-8000,at]).unwrap();let out=heatmap(&d,at).unwrap();let total:i64=out["heatmap"].as_array().unwrap().iter().map(|x|x["active_ms"].as_i64().unwrap()).sum();assert_eq!(total,10000);assert_eq!(out["truncated"],false);assert!(out["ambiguous_overlaps"].as_u64().unwrap()>0);}
}

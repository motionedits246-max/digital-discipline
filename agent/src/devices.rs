//! Explicit, offline, user-operated cross-device record exchange.
//! No automatic discovery, public server, cross-device enforcement or cloud sync.
//! Encryption/decryption of portable bundles is handled in the Tauri WebView;
//! the agent accepts ONLY already-decrypted records over its authenticated loopback IPC.
use super::{AppState,ApiResult,err,now_ms};
use axum::{extract::State,http::StatusCode,Json};
use rusqlite::{params,Connection};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::{collections::BTreeMap,sync::Arc};
use uuid::Uuid;

const MAX_RECORDS:usize=1000;
const WINDOW_MS:i64=30*86_400_000;

pub fn migrate(db:&Connection)->rusqlite::Result<()> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS device_sessions (
        device_id TEXT NOT NULL,session_id TEXT NOT NULL,start_ms INTEGER NOT NULL,
        end_ms INTEGER NOT NULL,app TEXT NOT NULL,idle INTEGER NOT NULL,domain TEXT,
        PRIMARY KEY(device_id,session_id));
        CREATE INDEX IF NOT EXISTS idx_remote_session_window ON device_sessions(start_ms,end_ms);
        CREATE TABLE IF NOT EXISTS linked_devices(device_id TEXT PRIMARY KEY,display_name TEXT NOT NULL,last_import_ms INTEGER NOT NULL);")?;
    if super::features::read_setting(db,"device_id").is_empty(){
        db.execute("INSERT OR IGNORE INTO settings(key,value) VALUES('device_id',?1)",params![Uuid::new_v4().to_string()])?;
    }
    Ok(())
}

#[derive(Deserialize,Serialize)]
#[serde(deny_unknown_fields)]
pub struct Record{pub session_id:String,pub start_ms:i64,pub end_ms:i64,pub app:String,pub idle:bool,pub domain:Option<String>}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import{pub schema:String,pub source_device_id:String,pub source_device_name:String,pub records:Vec<Record>}

pub async fn export(State(s):State<Arc<AppState>>)->ApiResult {
    let db=s.db.lock().unwrap();
    let dev=super::features::read_setting(&db,"device_id");
    let mut q=db.prepare("SELECT id,start_ms,end_ms,app,idle,domain FROM sessions WHERE end_ms>?1 AND end_ms>start_ms ORDER BY start_ms DESC LIMIT ?2")
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let cutoff=now_ms()-WINDOW_MS;
    let records=q.query_map(params![cutoff,MAX_RECORDS as i64+1],|r|Ok(json!({"session_id":r.get::<_,String>(0)?,"start_ms":r.get::<_,i64>(1)?,"end_ms":r.get::<_,i64>(2)?,"app":r.get::<_,String>(3)?,"idle":r.get::<_,i64>(4)?>0,"domain":r.get::<_,Option<String>>(5)?})))
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let truncated=records.len()>MAX_RECORDS;
    Ok(Json(json!({"ok":true,"schema":"dd-device-export-v1","source_device_id":dev,
        "source_device_name":std::env::var("COMPUTERNAME").unwrap_or_else(|_|"Windows device".into()).chars().take(60).collect::<String>(),
        "created_ms":now_ms(),"truncated":truncated,"records":records.into_iter().take(MAX_RECORDS).collect::<Vec<_>>(),
        "notice":"Only local sessions. At most 1000 recent records, never an archival backup."})))
}

fn valid_name(s:&str)->bool{!s.is_empty()&&s.len()<=80&&!s.chars().any(char::is_control)}
pub(crate) fn record_valid(r:&Record,now:i64)->bool{
    Uuid::parse_str(&r.session_id).is_ok() && r.end_ms>r.start_ms && r.start_ms>now-WINDOW_MS-3_600_000 &&
    r.start_ms<=now+300_000 && r.end_ms<=now+300_000 && r.end_ms-r.start_ms<=WINDOW_MS &&
    !r.app.is_empty() && r.app.len()<=120 && r.app.chars().all(|c|!c.is_control() && c!='/' && c!='\\') &&
    r.domain.as_ref().is_none_or(|d|crate::domain::valid_domain(d))
}

pub async fn import(State(s):State<Arc<AppState>>,Json(p):Json<Import>)->ApiResult {
    let now=now_ms();
    if p.schema!="dd-device-export-v1" || Uuid::parse_str(&p.source_device_id).is_err() ||
        !valid_name(&p.source_device_name) || p.records.len()>MAX_RECORDS ||
        p.records.iter().any(|r|!record_valid(r,now)) {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_device_bundle"));
    }
    let mut db=s.db.lock().unwrap();
    if super::features::read_setting(&db,"device_id")==p.source_device_id {
        return Err(err(StatusCode::BAD_REQUEST,"cannot_import_own_device"));
    }
    let tx=db.transaction().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let mut inserted=0;
    for r in &p.records {
        if super::features::excluded(&tx,&r.app,r.domain.as_deref())||crate::data_lifecycle::rejection(&tx,r.start_ms,Some(&format!("app:{}",r.app.to_ascii_lowercase()))).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?.is_some()||r.domain.as_ref().is_some_and(|host|crate::data_lifecycle::rejection(&tx,r.start_ms,Some(&format!("domain:{host}"))).map_or(true,|r|r.is_some())){continue;}
        inserted+=tx.execute("INSERT OR IGNORE INTO device_sessions(device_id,session_id,start_ms,end_ms,app,idle,domain) VALUES(?1,?2,?3,?4,?5,?6,?7)",
            params![p.source_device_id,r.session_id,r.start_ms,r.end_ms,r.app,r.idle as u8,r.domain])
            .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    }
    tx.execute("INSERT INTO linked_devices(device_id,display_name,last_import_ms) VALUES(?1,?2,?3) ON CONFLICT(device_id) DO UPDATE SET display_name=excluded.display_name,last_import_ms=excluded.last_import_ms",
        params![p.source_device_id,p.source_device_name,now])
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    tx.commit().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"imported":inserted,"duplicates_or_excluded":p.records.len()-inserted,"note":"Remote records are report-only. They do not drive this device's restrictions."})))
}

pub async fn list(State(s):State<Arc<AppState>>)->ApiResult{
    let db=s.db.lock().unwrap();
    let mut q=db.prepare("SELECT device_id,display_name,last_import_ms FROM linked_devices ORDER BY last_import_ms DESC LIMIT 30")
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let devices=q.query_map([],|r|Ok(json!({"device_id":r.get::<_,String>(0)?,"display_name":r.get::<_,String>(1)?,"last_import_ms":r.get::<_,i64>(2)?})))
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?.collect::<rusqlite::Result<Vec<Value>>>()
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let cutoff=now_ms()-7*86_400_000;
    let mut per_device:BTreeMap<String,Vec<(i64,i64)>>=BTreeMap::new();
    for (table,col) in [("sessions","'local'"),("device_sessions","device_id")] {
        let sql=format!("SELECT {col},start_ms,end_ms FROM {table} WHERE idle=0 AND end_ms>?1 ORDER BY start_ms LIMIT 250000");
        let mut query=db.prepare(&sql).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
        let rows=query.query_map(params![cutoff],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,r.get::<_,i64>(2)?)))
            .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
        for row in rows {let (id,a,b)=row.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
            per_device.entry(id).or_default().push((a,b));}
    }
    let mut all=Vec::new();let mut breakdown=BTreeMap::new();
    for (device,intervals) in per_device {
        let amount=crate::policy::union_ms(intervals.clone(),cutoff,now_ms());
        breakdown.insert(device,amount);all.extend(intervals);
    }
    let total=crate::policy::union_ms(all,cutoff,now_ms());
    Ok(Json(json!({"ok":true,"own_device_id":super::features::read_setting(&db,"device_id"),"devices":devices,
        "observed_union_ms_7d":total,"per_device_observed_ms_7d":breakdown,
        "note":"Time is unioned across devices to avoid overlapping minutes. Device reports use encrypted imports or configured folder sync. Missing devices and hours are unknown."})))
}

pub async fn forget(State(s):State<Arc<AppState>>,axum::extract::Path(id):axum::extract::Path<String>)->ApiResult {
    if Uuid::parse_str(&id).is_err(){return Err(err(StatusCode::BAD_REQUEST,"invalid_device_id"));}
    let mut db=s.db.lock().unwrap();
    let tx=db.transaction().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    tx.execute("DELETE FROM device_sessions WHERE device_id=?1",params![id]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    tx.execute("DELETE FROM linked_devices WHERE device_id=?1",params![id]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    tx.execute("UPDATE sync_peers SET revoked=1 WHERE id=?1",params![id]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    tx.commit().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"forgotten":true})))
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn rejects_future_or_oversized_imports(){let now=1_800_000_000_000;
  let r=Record{session_id:Uuid::new_v4().to_string(),start_ms:now-1000,end_ms:now+500_000,app:"code.exe".into(),idle:false,domain:None};
  assert!(!record_valid(&r,now));
 }
}

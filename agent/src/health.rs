use crate::{AppState,ApiResult,err,now_ms};
use axum::{extract::State,http::StatusCode,Json};
use serde_json::json;
use std::{sync::{Arc,atomic::Ordering},fs};

pub async fn details(State(s):State<Arc<AppState>>)->ApiResult{
 let now=now_ms();
 let (last_event,event_count,session_count,auto,size)={
  let db=s.db.lock().unwrap();
  let last=db.query_row("SELECT MAX(occurred_ms) FROM raw_events",[],|r|r.get::<_,Option<i64>>(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"health_query_failed"))?;
  let count=db.query_row("SELECT COUNT(*) FROM raw_events",[],|r|r.get::<_,i64>(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"health_query_failed"))?;
  let sessions=db.query_row("SELECT COUNT(*) FROM sessions",[],|r|r.get::<_,i64>(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"health_query_failed"))?;
  let enabled=db.query_row("SELECT value FROM settings WHERE key='classification_auto'",[],|r|r.get::<_,String>(0)).unwrap_or_default()=="true";
  let bytes=db.path().and_then(|p|fs::metadata(p).ok().map(|m|m.len()));
  (last,count,sessions,enabled,bytes)
 };
 let browser=s.browser.lock().unwrap().clone();
 let (clock_rollbacks,observation_gaps)={let tracker=s.tracker.lock().unwrap();(tracker.reconciler.clock_rollbacks,tracker.reconciler.observation_gaps)};
 Ok(Json(json!({"ok":true,"sampled_at_ms":now,
  "agent":{"reachable":true,"version":env!("CARGO_PKG_VERSION"),"clock_rollbacks":clock_rollbacks,"observation_gaps":observation_gaps,"tracking_paused":s.paused.load(Ordering::SeqCst),
    "storage_last_error_at_ms":s.storage_last_error_ms.load(Ordering::SeqCst),
    "storage_error_count":s.storage_error_count.load(Ordering::SeqCst),
    "storage_degraded":s.storage_last_error_ms.load(Ordering::SeqCst)>now-120_000},
  "browser":{"last_message_at_ms":browser.last_event_ms,"last_rule_ack_at_ms":browser.rule_sync_ms,
   "recent_signal":browser.last_event_ms>0&&now-browser.last_event_ms<120_000,
   "recent_rule_ack":browser.rule_sync_ms>0&&now-browser.rule_sync_ms<180_000},
  "storage":{"db_bytes":size,"persisted_event_count":event_count,"session_rows":session_count,"last_accepted_event_at_ms":last_event},
  "optional_workers":{"auto_classification_enabled":auto},
  "redacted":true
 })))
}

//! History corrections are overlays. Raw observations remain available to the collector.
use crate::{category_labels, err, midnight_local_ms, now_ms, ApiResult, AppState};
use axum::{extract::{Path, State}, http::StatusCode, Json};
use chrono::{Duration, NaiveDate};
use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

const UNDO_MS: i64 = 30 * 86_400_000;

pub fn migrate(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS history_corrections(
           session_id TEXT PRIMARY KEY REFERENCES sessions(id) ON DELETE CASCADE,
           category TEXT, hidden INTEGER NOT NULL, updated_ms INTEGER NOT NULL);
         CREATE TABLE IF NOT EXISTS manual_activity(
           id TEXT PRIMARY KEY, start_ms INTEGER NOT NULL, end_ms INTEGER NOT NULL,
           label TEXT NOT NULL, category TEXT NOT NULL, created_ms INTEGER NOT NULL);
         CREATE INDEX IF NOT EXISTS idx_manual_activity_date ON manual_activity(start_ms);
         CREATE VIEW IF NOT EXISTS report_sessions AS
           SELECT s.*,c.category AS corrected_category FROM sessions s
           LEFT JOIN history_corrections c ON c.session_id=s.id WHERE COALESCE(c.hidden,0)=0;",
    )
}

fn storage(_: rusqlite::Error) -> (StatusCode, String) {
    err(StatusCode::INTERNAL_SERVER_ERROR, "history_storage_failed")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Correction {
    session_id: String,
    action: String,
    category: Option<String>,
}

pub async fn correct(State(state): State<Arc<AppState>>, Json(input): Json<Correction>) -> ApiResult {
    let db = state.db.lock().unwrap();
    if Uuid::parse_str(&input.session_id).is_err()
        || !matches!(input.action.as_str(), "undo" | "hide" | "category")
        || input.action=="category" && !input.category.as_deref().is_some_and(|value|category_labels::known(&db,value))
    {
        return Err(err(StatusCode::BAD_REQUEST, "invalid_correction"));
    }
    let now=now_ms();
    let changed=if input.action=="undo" {
        db.execute("DELETE FROM history_corrections WHERE session_id=?1 AND updated_ms>=?2",
            params![input.session_id,now-UNDO_MS])
    } else {
        db.execute(
            "INSERT INTO history_corrections(session_id,category,hidden,updated_ms)
             SELECT id,?2,?3,?4 FROM sessions WHERE id=?1 AND end_ms<=?4-10000
             ON CONFLICT(session_id) DO UPDATE SET category=excluded.category,
                 hidden=excluded.hidden,updated_ms=excluded.updated_ms",
            params![input.session_id,if input.action=="category"{input.category}else{None},
                input.action=="hide",now],
        )
    }.map_err(storage)?;
    if changed==0 {return Err(err(StatusCode::CONFLICT,"history_record_unavailable"));}
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManualInput {
    id: Option<String>,
    start_ms: Option<i64>,
    minutes: Option<u32>,
    label: Option<String>,
    category: Option<String>,
}

pub async fn manual(State(state): State<Arc<AppState>>, Json(input): Json<ManualInput>) -> ApiResult {
    let db = state.db.lock().unwrap();
    if let Some(id)=input.id {
        if Uuid::parse_str(&id).is_err(){return Err(err(StatusCode::BAD_REQUEST,"invalid_manual_record"));}
        db.execute("DELETE FROM manual_activity WHERE id=?1",[id]).map_err(storage)?;
        return Ok(Json(json!({"ok":true})));
    }
    let now=now_ms();
    let start=input.start_ms.ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_manual_time"))?;
    let minutes=input.minutes.filter(|value|(1..=480).contains(value))
        .ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_manual_duration"))?;
    let label=input.label.as_deref().map(str::trim).filter(|label|
        !label.is_empty() && label.chars().count()<=120 && !label.chars().any(char::is_control))
        .ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_manual_label"))?;
    let category=input.category.as_deref().filter(|value|category_labels::known(&db,value))
        .ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_manual_category"))?;
    if start<now-365*86_400_000 || start>now-minutes as i64*60_000 {
        return Err(err(StatusCode::BAD_REQUEST,"manual_time_must_be_past"));
    }
    db.execute("INSERT INTO manual_activity VALUES(?1,?2,?3,?4,?5,?6)",
        params![Uuid::new_v4().to_string(),start,start+minutes as i64*60000,label,category,now])
        .map_err(storage)?;
    Ok(Json(json!({"ok":true})))
}

pub async fn list(Path(day): Path<String>, State(state): State<Arc<AppState>>) -> ApiResult {
    let date=NaiveDate::parse_from_str(&day,"%Y-%m-%d").ok()
        .filter(|day|day.to_string().len()==10)
        .ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_date"))?;
    let start=midnight_local_ms(date);
    let end=midnight_local_ms(date+Duration::days(1));
    let db=state.db.lock().unwrap();
    let now=now_ms();
    let mut q=db.prepare("SELECT c.session_id,s.app,c.category,c.hidden,c.updated_ms
        FROM history_corrections c JOIN sessions s ON c.session_id=s.id
        WHERE s.end_ms>?1 AND s.start_ms<?2 ORDER BY c.updated_ms DESC LIMIT 250").map_err(storage)?;
    let corrections=q.query_map(params![start,end],|r|Ok(json!({"session_id":r.get::<_,String>(0)?,
        "app":r.get::<_,String>(1)?,"category":r.get::<_,Option<String>>(2)?,
        "hidden":r.get::<_,bool>(3)?,"can_undo":r.get::<_,i64>(4)? >= now-UNDO_MS})))
        .map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
    let mut q=db.prepare("SELECT id,start_ms,end_ms,label,category FROM manual_activity
        WHERE end_ms>?1 AND start_ms<?2 ORDER BY start_ms LIMIT 250").map_err(storage)?;
    let manual=q.query_map(params![start,end],|r|Ok(json!({"id":r.get::<_,String>(0)?,"start_ms":r.get::<_,i64>(1)?,
        "end_ms":r.get::<_,i64>(2)?,"label":r.get::<_,String>(3)?,"category":r.get::<_,String>(4)?})))
        .map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
    Ok(Json(json!({"ok":true,"corrections":corrections,"manual":manual,
        "may_be_truncated":corrections.len()==250||manual.len()==250})))
}

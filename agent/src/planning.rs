//! Owner-entered plans and weekly targets. Measurement comes from stored observations.
use crate::{err, features, midnight_local_ms, now_ms, policy, ApiResult, AppState};
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::BTreeSet, sync::Arc};
use uuid::Uuid;

pub fn migrate(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS planner_blocks(
            id TEXT PRIMARY KEY, day TEXT NOT NULL, start_min INTEGER NOT NULL,
            minutes INTEGER NOT NULL, kind TEXT NOT NULL, title TEXT NOT NULL,
            profile_id TEXT REFERENCES workflow_profiles(id) ON DELETE SET NULL,
            status TEXT NOT NULL DEFAULT 'planned', focus_id TEXT, revision INTEGER NOT NULL);
         CREATE INDEX IF NOT EXISTS idx_planner_day ON planner_blocks(day,start_min);
         CREATE TABLE IF NOT EXISTS weekly_targets(
            id TEXT PRIMARY KEY, title TEXT NOT NULL, metric TEXT NOT NULL,
            target INTEGER NOT NULL, days INTEGER NOT NULL, revision INTEGER NOT NULL);",
    )
}

fn storage(_: rusqlite::Error) -> (StatusCode, String) {
    err(StatusCode::INTERNAL_SERVER_ERROR, "planner_storage_failed")
}

fn valid_label(text: &str) -> bool {
    !text.trim().is_empty()
        && text.chars().count() <= 120
        && !text.chars().any(char::is_control)
}

fn date(value: &str) -> Option<NaiveDate> {
    let day = NaiveDate::parse_from_str(value, "%Y-%m-%d").ok()?;
    (day.to_string() == value && (2000..=2100).contains(&day.year())).then_some(day)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockInput {
    id: Option<String>,
    revision: Option<i64>,
    day: String,
    start_min: u32,
    minutes: u32,
    kind: String,
    title: String,
    profile_id: Option<String>,
}

pub async fn save_block(State(state): State<Arc<AppState>>, Json(input): Json<BlockInput>) -> ApiResult {
    let day = date(&input.day).ok_or_else(|| err(StatusCode::BAD_REQUEST, "invalid_plan_day"))?;
    let today = Local::now().date_naive();
    if day < today - Duration::days(365) || day > today + Duration::days(365)
        || !valid_label(&input.title)
        || !matches!(input.kind.as_str(), "focus" | "task" | "break" | "free")
        || !(1..=240).contains(&input.minutes)
        || input.start_min > 1439
        || input.start_min + input.minutes > 1440
        || input.kind != "focus" && input.profile_id.is_some()
        || input.id.as_ref().is_some_and(|id| Uuid::parse_str(id).is_err())
    {
        return Err(err(StatusCode::BAD_REQUEST, "invalid_plan_block"));
    }
    let db = state.db.lock().unwrap();
    if let Some(profile) = &input.profile_id {
        let available: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM workflow_profiles WHERE id=?1 AND archived=0)",
            [profile], |r| r.get(0),
        ).map_err(storage)?;
        if !available {
            return Err(err(StatusCode::BAD_REQUEST, "profile_not_found"));
        }
    }
    let id = input.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    if input.id.is_some() {
        let changed = db.execute(
            "UPDATE planner_blocks SET day=?2,start_min=?3,minutes=?4,kind=?5,title=?6,
             profile_id=?7,revision=revision+1 WHERE id=?1 AND revision=?8 AND status='planned'",
            params![id, input.day, input.start_min, input.minutes, input.kind, input.title.trim(),
                input.profile_id, input.revision],
        ).map_err(storage)?;
        if changed == 0 { return Err(err(StatusCode::CONFLICT, "plan_changed")); }
    } else {
        db.execute(
            "INSERT INTO planner_blocks(id,day,start_min,minutes,kind,title,profile_id,revision)
             VALUES(?1,?2,?3,?4,?5,?6,?7,1)",
            params![id,input.day,input.start_min,input.minutes,input.kind,input.title.trim(),input.profile_id],
        ).map_err(storage)?;
    }
    Ok(Json(json!({"ok":true,"id":id})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockAction {
    id: String,
    revision: i64,
    action: String,
}

pub async fn act(State(state): State<Arc<AppState>>, Json(input): Json<BlockAction>) -> ApiResult {
    if Uuid::parse_str(&input.id).is_err() || !matches!(input.action.as_str(), "start" | "skip" | "remove") {
        return Err(err(StatusCode::BAD_REQUEST, "invalid_plan_action"));
    }
    let mut db = state.db.lock().unwrap();
    let tx = db.transaction().map_err(storage)?;
    let block = tx.query_row(
        "SELECT kind,minutes,profile_id,status,revision FROM planner_blocks WHERE id=?1",
        [&input.id],
        |r| Ok((r.get::<_, String>(0)?,r.get::<_, u32>(1)?,r.get::<_, Option<String>>(2)?,
            r.get::<_, String>(3)?,r.get::<_, i64>(4)?)),
    ).optional().map_err(storage)?.ok_or_else(|| err(StatusCode::NOT_FOUND,"plan_not_found"))?;
    if block.4 != input.revision || block.3 == "started" {
        return Err(err(StatusCode::CONFLICT, "plan_changed"));
    }
    let mut focus_id = None;
    match input.action.as_str() {
        "start" if block.0 == "focus" && block.3 == "planned" => {
            let focus = features::start_planned_focus(&tx, block.1, block.2, now_ms())?;
            focus_id = focus["id"].as_str().map(str::to_owned);
            tx.execute(
                "UPDATE planner_blocks SET status='started',focus_id=?2,revision=revision+1 WHERE id=?1",
                params![input.id,focus_id],
            ).map_err(storage)?;
        }
        "skip" if block.3 == "planned" => {
            tx.execute("UPDATE planner_blocks SET status='skipped',revision=revision+1 WHERE id=?1",
                [&input.id]).map_err(storage)?;
        }
        "remove" => { tx.execute("DELETE FROM planner_blocks WHERE id=?1", [&input.id]).map_err(storage)?; }
        _ => return Err(err(StatusCode::CONFLICT, "plan_action_unavailable")),
    }
    tx.commit().map_err(storage)?;
    Ok(Json(json!({"ok":true,"focus_id":focus_id})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetInput {
    id: Option<String>,
    revision: Option<i64>,
    title: String,
    metric: String,
    target: u32,
    days: u8,
    #[serde(default)]
    remove: bool,
}

pub async fn save_target(State(state): State<Arc<AppState>>, Json(input): Json<TargetInput>) -> ApiResult {
    if input.id.as_ref().is_some_and(|id| Uuid::parse_str(id).is_err()) {
        return Err(err(StatusCode::BAD_REQUEST, "invalid_weekly_target"));
    }
    let db = state.db.lock().unwrap();
    if input.remove {
        if db.execute("DELETE FROM weekly_targets WHERE id=?1 AND revision=?2",
            params![input.id,input.revision]).map_err(storage)? == 0 {
            return Err(err(StatusCode::CONFLICT,"target_changed"));
        }
        return Ok(Json(json!({"ok":true})));
    }
    let max = match input.metric.as_str() {
        "observed_minutes" | "reading_minutes" => 10080,
        "completed_tasks" => 1000,
        _ => 0,
    };
    if !valid_label(&input.title) || !(1..=max).contains(&input.target) || !(1..=127).contains(&input.days) {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_weekly_target"));
    }
    let id = input.id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());
    if input.id.is_some() {
        if db.execute(
            "UPDATE weekly_targets SET title=?2,metric=?3,target=?4,days=?5,revision=revision+1 WHERE id=?1 AND revision=?6",
            params![id,input.title.trim(),input.metric,input.target,input.days,input.revision],
        ).map_err(storage)? == 0 { return Err(err(StatusCode::CONFLICT,"target_changed")); }
    } else {
        db.execute("INSERT INTO weekly_targets VALUES(?1,?2,?3,?4,?5,1)",
            params![id,input.title.trim(),input.metric,input.target,input.days]).map_err(storage)?;
    }
    Ok(Json(json!({"ok":true,"id":id})))
}

fn measured(db: &Connection, start: i64, end: i64) -> rusqlite::Result<(i64, BTreeSet<u32>)> {
    let mut query = db.prepare(
        "SELECT start_ms,end_ms,idle FROM report_sessions WHERE end_ms>?1 AND start_ms<?2 ORDER BY start_ms",
    )?;
    let mut ranges = Vec::new();
    let mut coverage = BTreeSet::new();
    let mut rows = query.query(params![start,end])?;
    while let Some(row) = rows.next()? {
        let lo = row.get::<_, i64>(0)?.max(start);
        let hi = row.get::<_, i64>(1)?.min(end);
        if hi <= lo { continue; }
        if row.get::<_, i64>(2)? == 0 { ranges.push((lo,hi)); }
        if let Some(timestamp) = chrono::DateTime::from_timestamp_millis(lo) {
            let mut day = timestamp.with_timezone(&Local).date_naive();
            while midnight_local_ms(day) < hi {
                coverage.insert(day.weekday().num_days_from_monday());
                day += Duration::days(1);
            }
        }
    }
    Ok((policy::union_ms(ranges,start,end),coverage))
}

pub fn report(db: &Connection, now: i64) -> rusqlite::Result<Value> {
    let local = chrono::DateTime::from_timestamp_millis(now)
        .ok_or(rusqlite::Error::InvalidQuery)?.with_timezone(&Local);
    let today = local.date_naive();
    let monday = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let start = midnight_local_ms(monday);
    let prior = midnight_local_ms(monday - Duration::days(7));
    let prior_end = local.naive_local().checked_sub_signed(Duration::days(7))
        .and_then(|value| value.and_local_timezone(Local).earliest())
        .map_or(prior, |value| value.timestamp_millis());
    let (active, covered) = measured(db,start,now)?;
    let (previous, previous_covered) = measured(db,prior,prior_end)?;
    let completed: u32 = db.query_row(
        "SELECT count(*) FROM workflow_tasks WHERE state='completed' AND finished_ms>=?1 AND finished_ms<?2",
        params![start,now],|r|r.get(0),
    )?;
    let interventions: u32 = db.query_row(
        "SELECT count(*) FROM audits WHERE occurred_ms>=?1 AND occurred_ms<?2 AND kind='rule_transition'
         AND json_extract(detail,'$.triggered')=1", params![start,now],|r|r.get(0),
    )?;
    let exceptions: u32 = db.query_row(
        "SELECT count(*) FROM temporary_exceptions WHERE created_ms>=?1 AND created_ms<?2",
        params![start,now],|r|r.get(0),
    )?;
    let focus_sessions: u32 = db.query_row(
        "SELECT count(*) FROM focus_sessions WHERE start_ms>=?1 AND start_ms<?2
         AND (ended_ms IS NOT NULL OR end_ms<=?2)
         AND NOT EXISTS(SELECT 1 FROM focus_cycles WHERE focus_id=focus_sessions.id AND state='paused')",
        params![start,now],|r|r.get(0),
    )?;
    let reading=crate::attention::reading_ms(db,start,now)?;
    let video:i64=db.query_row("SELECT COALESCE(SUM(watched_ms),0) FROM videos
        WHERE platform='youtube_watch' AND qualified=1 AND occurred_ms>=?1 AND occurred_ms<?2",
        params![start,now],|r|r.get(0))?;
    let mut query = db.prepare("SELECT id,title,metric,target,days,revision FROM weekly_targets ORDER BY title LIMIT 100")?;
    let targets = query.query_map([],|r|{
        let metric:String=r.get(2)?;
        let target:u32=r.get(3)?;
        let days:u8=r.get(4)?;
        let progress=match metric.as_str() {
            "completed_tasks"=>completed,"reading_minutes"=>(reading/60000) as u32,_=>(active/60000) as u32,
        };
        let remaining=target.saturating_sub(progress);
        let remaining_days=(today.weekday().num_days_from_monday()..7)
            .filter(|day|days & (1 << day)!=0).count() as u32;
        Ok(json!({"id":r.get::<_,String>(0)?,"title":r.get::<_,String>(1)?,"metric":metric,
            "target":target,"days":days,"revision":r.get::<_,i64>(5)?,"progress":progress,
            "remaining":remaining,"remaining_days":remaining_days,
            "per_day":if remaining_days>0{Some(remaining.div_ceil(remaining_days))}else{None}}))
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(json!({"start":monday.to_string(),"through":today.to_string(),"observed_ms":active,
        "previous_observed_ms":previous,"days_observed":covered.len(),
        "previous_days_observed":previous_covered.len(),
        "comparison_ms":if !covered.is_empty() && covered==previous_covered {Some(active-previous)}else{None},
        "reading_ms":reading,"regular_video_ms":video,"completed_tasks":completed,"focus_sessions_ended":focus_sessions,
        "rule_activations":interventions,"temporary_exceptions":exceptions,"targets":targets}))
}

pub async fn list(State(state): State<Arc<AppState>>) -> ApiResult {
    let path = state.db.lock().unwrap().path().map(str::to_owned)
        .ok_or_else(||err(StatusCode::SERVICE_UNAVAILABLE,"planner_unavailable"))?;
    let value = tokio::task::spawn_blocking(move|| -> rusqlite::Result<Value> {
        let db=Connection::open_with_flags(path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
        db.busy_timeout(std::time::Duration::from_secs(2))?;
        let tx=db.unchecked_transaction()?;
        let now=now_ms();
        let today=Local::now().date_naive();
        let mut query=tx.prepare("SELECT b.id,b.day,b.start_min,b.minutes,b.kind,b.title,b.profile_id,b.status,b.revision,
            EXISTS(SELECT 1 FROM planner_blocks a WHERE a.day=b.day AND a.id<>b.id AND a.status='planned'
                AND a.start_min < b.start_min+b.minutes AND b.start_min < a.start_min+a.minutes)
            FROM planner_blocks b WHERE b.day BETWEEN ?1 AND ?2 ORDER BY b.day,b.start_min LIMIT 500")?;
        let blocks=query.query_map(params![(today-Duration::days(7)).to_string(),(today+Duration::days(30)).to_string()], |r|{
            let day:String=r.get(1)?;
            let start:u32=r.get(2)?;
            let minutes:u32=r.get(3)?;
            let status:String=r.get(7)?;
            let missed=status=="planned" && date(&day).is_some_and(|day| {
                let local=Local::now();
                day<today || day==today && chrono::Timelike::hour(&local)*60+chrono::Timelike::minute(&local)>=start+minutes
            });
            Ok(json!({"id":r.get::<_,String>(0)?,"day":day,"start_min":start,"minutes":minutes,
                "kind":r.get::<_,String>(4)?,"title":r.get::<_,String>(5)?,"profile_id":r.get::<_,Option<String>>(6)?,
                "status":status,"revision":r.get::<_,i64>(8)?,"overlaps":r.get::<_,bool>(9)?,"missed":missed}))
        })?.collect::<rusqlite::Result<Vec<_>>>()?;
        let report=report(&tx,now)?;
        Ok(json!({"ok":true,"today":today.to_string(),"blocks":blocks,"report":report,
            "may_be_truncated":blocks.len()==500}))
    }).await.map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"planner_worker_failed"))?.map_err(storage)?;
    Ok(Json(value))
}

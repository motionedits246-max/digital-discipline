//! Opt-in reading, shared quiet hours, and a short-form intervention progression.
use crate::{err, features, now_ms, policy, ApiResult, AppState};
use axum::{extract::State,http::StatusCode,Json};
use chrono::{Local, Timelike};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize,Serialize};
use serde_json::{json,Value};
use std::sync::Arc;
use uuid::Uuid;

const PRESENCE_MS: i64 = 180_000;
pub fn migrate(db:&Connection)->rusqlite::Result<()> {
    db.execute_batch("CREATE TABLE IF NOT EXISTS reading_windows(
        id TEXT PRIMARY KEY, app TEXT NOT NULL, start_ms INTEGER NOT NULL, end_ms INTEGER NOT NULL,
        confirmed_ms INTEGER NOT NULL, stopped INTEGER NOT NULL DEFAULT 0);
      CREATE TABLE IF NOT EXISTS reading_segments(
        id INTEGER PRIMARY KEY, window_id TEXT REFERENCES reading_windows(id) ON DELETE CASCADE,
        start_ms INTEGER NOT NULL, end_ms INTEGER NOT NULL);
      CREATE TABLE IF NOT EXISTS intervention_state(
        id INTEGER PRIMARY KEY CHECK(id=1), level INTEGER NOT NULL, policy_until_ms INTEGER NOT NULL,
        snoozed_until_ms INTEGER NOT NULL, last_play_ms INTEGER NOT NULL);")
}
fn storage(_:rusqlite::Error)->(StatusCode,String) {
    err(StatusCode::INTERNAL_SERVER_ERROR,"attention_storage_failed")
}
fn store<T:Serialize>(db:&Connection,key:&str,value:&T)->rusqlite::Result<()> {
    db.execute("INSERT OR REPLACE INTO settings(key,value) VALUES(?1,?2)",
        params![key,serde_json::to_string(value).map_err(|_|rusqlite::Error::InvalidQuery)?])?;
    Ok(())
}
fn load<T:for<'a> Deserialize<'a>+Default>(db:&Connection,key:&str)->T {
    serde_json::from_str(&features::read_setting(db,key)).unwrap_or_default()
}

#[derive(Default,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QuietHours {dnd:bool, start:Option<u16>, end:Option<u16>}
fn quiet_at(settings:&QuietHours,minute:u16)->bool {
    settings.dnd || match (settings.start,settings.end) {
        (Some(a),Some(b)) if a<b => (a..b).contains(&minute),
        (Some(a),Some(b)) if a>b => minute>=a || minute<b,
        _ => false,
    }
}
pub fn muted(db:&Connection)->bool {
    let local=Local::now();
    quiet_at(&load(db,"shared_quiet_hours"),(local.hour()*60+local.minute()) as u16)
}
pub async fn quiet(State(state):State<Arc<AppState>>,Json(input):Json<QuietHours>)->ApiResult {
    if input.start.is_some()!=input.end.is_some() ||
        input.start.into_iter().chain(input.end).any(|time|time>=1440) {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_quiet_hours"));
    }
    store(&state.db.lock().unwrap(),"shared_quiet_hours",&input).map_err(storage)?;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReadingAction {action:String, app:Option<String>, minutes:Option<u32>}
fn current_reading(db:&Connection,now:i64)->rusqlite::Result<Option<(String,String,i64,i64,i64)>> {
    db.query_row("SELECT id,app,start_ms,end_ms,confirmed_ms FROM reading_windows
        WHERE stopped=0 AND end_ms>=?1 AND confirmed_ms>?2 ORDER BY start_ms DESC LIMIT 1",
        params![now,now-PRESENCE_MS], |r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?))).optional()
}
pub async fn reading(State(state):State<Arc<AppState>>,Json(input):Json<ReadingAction>)->ApiResult {
    let now=now_ms();
    let mut db=state.db.lock().unwrap();
    let tx=db.transaction().map_err(storage)?;
    match input.action.as_str() {
        "start" => {
            let app=input.app.as_deref().filter(|app|crate::inbox::safe_native_app(app))
                .ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_reading_app"))?;
            let minutes=input.minutes.filter(|minutes|[5,10,20,30].contains(minutes))
                .ok_or_else(||err(StatusCode::BAD_REQUEST,"invalid_reading_duration"))?;
            if crate::data_lifecycle::is_private(&tx,now) || state.paused.load(std::sync::atomic::Ordering::SeqCst) {
                return Err(err(StatusCode::CONFLICT,"resume_recording_to_start_reading"));
            }
            tx.execute("UPDATE reading_windows SET stopped=1 WHERE stopped=0",[]).map_err(storage)?;
            tx.execute("INSERT INTO reading_windows VALUES(?1,?2,?3,?4,?3,0)",
                params![Uuid::new_v4().to_string(),app,now,now+minutes as i64*60000]).map_err(storage)?;
        }
        "confirm" => {
            let Some((id,app,_,_,previous))=current_reading(&tx,now).map_err(storage)? else {
                return Err(err(StatusCode::CONFLICT,"reading_confirmation_expired"));
            };
            if now-previous<15_000 {
                return Err(err(StatusCode::BAD_REQUEST,"reading_confirmation_too_soon"));
            }
            if crate::data_lifecycle::is_private(&tx,now) || features::excluded(&tx,&app,None) {
                return Err(err(StatusCode::CONFLICT,"reading_recording_unavailable"));
            }
            let ranges={
                let mut q=tx.prepare("SELECT start_ms,end_ms FROM sessions WHERE lower(app)=?1 AND end_ms>?2
                    AND start_ms<?3 ORDER BY start_ms,id").map_err(storage)?;
                let rows=q.query_map(params![app,previous,now],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?)))
                    .map_err(storage)?;
                rows.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?
            };
            let mut covered=previous;
            for (start,end) in ranges {
                let start=start.max(covered);
                let end=end.min(now);
                if end>start {
                    tx.execute("INSERT INTO reading_segments(window_id,start_ms,end_ms) VALUES(?1,?2,?3)",
                        params![id,start,end]).map_err(storage)?;
                    covered=end;
                }
            }
            tx.execute("UPDATE reading_windows SET confirmed_ms=?2 WHERE id=?1",params![id,now]).map_err(storage)?;
        }
        "stop" => {tx.execute("UPDATE reading_windows SET stopped=1 WHERE stopped=0",[]).map_err(storage)?;}
        _ => return Err(err(StatusCode::BAD_REQUEST,"invalid_reading_action")),
    }
    tx.commit().map_err(storage)?;
    Ok(Json(json!({"ok":true})))
}
pub fn reading_ms(db:&Connection,start:i64,end:i64)->rusqlite::Result<i64> {
    let mut q=db.prepare("SELECT start_ms,end_ms FROM reading_segments WHERE end_ms>?1 AND start_ms<?2")?;
    let ranges=q.query_map(params![start,end],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?)))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(policy::union_ms(ranges,start,end))
}

#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progression {
    enabled:bool, platform:String, window_minutes:u32, stages:[u32;4], block_minutes:u32,
}
impl Default for Progression {
    fn default()->Self {Self{enabled:false,platform:"youtube_shorts".into(),window_minutes:15,stages:[3,6,9,12],block_minutes:5}}
}
pub async fn configure(State(state):State<Arc<AppState>>,Json(input):Json<Progression>)->ApiResult {
    if !matches!(input.platform.as_str(),"youtube_shorts"|"instagram_reels")
        || !(3..=60).contains(&input.window_minutes) || !(1..=30).contains(&input.block_minutes)
        || input.stages[0]==0 || input.stages[3]>100 || !input.stages.windows(2).all(|pair|pair[0]<pair[1])
    {return Err(err(StatusCode::BAD_REQUEST,"invalid_intervention_settings"));}
    let db=state.db.lock().unwrap();
    let tx=db.unchecked_transaction().map_err(storage)?;
    store(&tx,"intervention_terms",&input).map_err(storage)?;
    tx.execute("DELETE FROM intervention_state",[]).map_err(storage)?;
    tx.execute("INSERT OR REPLACE INTO settings VALUES('intervention_started_ms',?1)",[now_ms().to_string()]).map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(Json(json!({"ok":true})))
}
pub async fn snooze(State(state):State<Arc<AppState>>)->ApiResult {
    state.db.lock().unwrap().execute("UPDATE intervention_state SET snoozed_until_ms=?1 WHERE id=1",[now_ms()+600000]).map_err(storage)?;
    Ok(Json(json!({"ok":true})))
}
fn progression(db:&Connection,now:i64)->rusqlite::Result<(Value,Vec<Value>,Option<i64>)> {
    let settings:Progression=load(db,"intervention_terms");
    if !settings.enabled {return Ok((json!({"enabled":false,"level":0}),vec![],None));}
    let window=settings.window_minutes as i64*60_000;
    let since=features::read_setting(db,"intervention_started_ms").parse::<i64>().unwrap_or(now);
    let (count,last):(u32,i64)=db.query_row("SELECT count(*),COALESCE(MAX(occurred_ms),0) FROM videos
        WHERE platform=?1 AND qualified=1 AND occurred_ms>?2 AND occurred_ms<=?3",
        params![settings.platform,(now-window).max(since),now],|r|Ok((r.get(0)?,r.get(1)?)))?;
    let (mut level,mut until,snoozed,old_last):(usize,i64,i64,i64)=db.query_row(
        "SELECT level,policy_until_ms,snoozed_until_ms,last_play_ms FROM intervention_state WHERE id=1",
        [],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?.unwrap_or((0,0,0,0));
    if old_last>0 && (count==0 && until<=now || last>old_last+window) {level=0;until=0;}
    let observed=settings.stages.iter().filter(|&&threshold|count>=threshold).count();
    let mut next_check=None;
    if observed>level && snoozed<=now && !crate::data_lifecycle::is_private(db,now) {
        level=observed;
        if level==4 {until=now+settings.block_minutes as i64*60_000;}
        if level>=2 {
            let label=match level {2=>"Short-form reminder",3=>"Feed reduction enabled",_=>"Timed short-feed limit"};
            crate::inbox::record(db,&format!("intervention:{level}:{last}"),"intervention",label,
                json!({"stage":level,"qualified_plays":count}),"Inbox",None,now)?;
        }
    }
    db.execute("INSERT INTO intervention_state VALUES(1,?1,?2,?3,?4)
        ON CONFLICT(id) DO UPDATE SET level=excluded.level,policy_until_ms=excluded.policy_until_ms,last_play_ms=excluded.last_play_ms",
        params![level,until,snoozed,last.max(old_last)])?;
    let host=if settings.platform=="youtube_shorts"{"www.youtube.com"}else{"www.instagram.com"};
    let path=if settings.platform=="youtube_shorts"{"/shorts"}else{"/reels"};
    let mut blocks=Vec::new();
    if snoozed>now {next_check=Some(snoozed);}
    else if (level==3&&observed>=3)||(level==4&&until>now) {
        let effective=if level==4{until}else{last+window};
        let expiry=effective.min(now+180000);
        blocks.push(json!({"id":"gentle-short-feed","domain":host,
            "path_prefix":if level==4{Some(path)}else{None},
            "action":if level==4{"block"}else{"hide_feed"},
            "reason":"Your gradual short-feed rule","expires_at_ms":expiry}));
        next_check=Some(expiry);
    }
    Ok((json!({"enabled":true,"level":level,"plays":count,"window_minutes":settings.window_minutes,
        "snoozed_until_ms":snoozed,"restriction_until_ms":until}),blocks,next_check))
}
pub fn pill_hint(db:&Connection,now:i64)->bool {
    let settings:Progression=load(db,"intervention_terms");
    if !settings.enabled || muted(db) || crate::data_lifecycle::is_private(db,now) {
        return false;
    }
    db.query_row(
        "SELECT EXISTS(SELECT 1 FROM intervention_state WHERE level=1 AND snoozed_until_ms<=?1 AND last_play_ms>?2)",
        params![now, now-settings.window_minutes as i64*60_000],
        |row|row.get(0),
    ).unwrap_or(false)
}
pub fn browser_policy(db:&Connection,now:i64)->rusqlite::Result<(Vec<Value>,Option<i64>)> {
    let (_,blocks,next)=progression(db,now)?;
    Ok((blocks,next))
}
pub async fn status(State(state):State<Arc<AppState>>)->ApiResult {
    let db=state.db.lock().unwrap();
    let now=now_ms();
    let (intervention,_,_)=progression(&db,now).map_err(storage)?;
    let reading=current_reading(&db,now).map_err(storage)?.map(|(id,app,start,end,confirmed)|json!({
        "id":id,"app":app,"start_ms":start,"end_ms":end,"confirm_by_ms":(confirmed+PRESENCE_MS).min(end)
    }));
    Ok(Json(json!({"ok":true,"reading":reading,"quiet":load::<QuietHours>(&db,"shared_quiet_hours"),
        "muted":muted(&db),"intervention":intervention,"terms":load::<Progression>(&db,"intervention_terms")})))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn quiet_hours_wrap_midnight_and_do_not_latch() {
        let settings=QuietHours{dnd:false,start:Some(22*60),end:Some(7*60)};
        assert!(quiet_at(&settings,23*60));
        assert!(quiet_at(&settings,6*60));
        assert!(!quiet_at(&settings,7*60));
        assert!(!quiet_at(&settings,12*60));
    }
}

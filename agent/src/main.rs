// Keep CONSOLE subsystem so Chromium native messaging inherits valid stdin/stdout pipes.
// Background launches hide the window using CREATE_NO_WINDOW on Windows.
mod domain;
mod native;
mod bridge;
mod credentials;
mod workflow;
mod health;
mod windows;
mod policy;
mod insights;
mod analytics;
mod features;
mod classification;
mod classification_store;
mod companion;
mod service_discovery;
mod schema;
mod tracking;
mod reporting;
mod data_lifecycle;
mod export_snapshot;
mod focus_workflow;
mod policy_exceptions;
mod inbox;
mod prediction;
mod devices;
mod hourly;
#[cfg(test)]
mod ingestion_tests;

use axum::{extract::{Path, State, Request, DefaultBodyLimit}, http::{header::AUTHORIZATION, StatusCode}, middleware::{self, Next}, response::{IntoResponse, Response}, routing::{delete, get, post}, Json, Router};
use chrono::{Duration, Local, NaiveDate, TimeZone, Utc};
use domain::{Observation, Reconciler};
use rand::RngCore;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{fs, path::PathBuf, sync::{atomic::{AtomicBool, AtomicI64, AtomicU64, Ordering}, Arc, Mutex}, thread, time::Duration as StdDuration};
use uuid::Uuid;

type ApiResult = Result<Json<Value>, (StatusCode, String)>;

struct Tracker { reconciler: Reconciler, session_id: Option<String> }
#[derive(Default, Clone)]
struct BrowserStatus { domain: Option<String>, profile_id:Option<String>, browser: String, focused: bool, last_event_ms: i64, rule_sync_ms: i64, rule_count: u32 }
struct AppState {
    db: Mutex<Connection>, tracker: Mutex<Tracker>, browser: Mutex<BrowserStatus>,
    paused: AtomicBool, stopping: AtomicBool, storage_last_error_ms: AtomicI64, storage_error_count:AtomicU64, token: String, shutdown: Arc<tokio::sync::Notify>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BrowserPayload { browser: String, domain: Option<String>, focused: bool, #[serde(default)] profile_id:Option<String> }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VideoPayload { platform: String, video_id: String, instance_id: String, watched_ms: i64 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BlockAck { applied_count: u32 }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VisitPayload {platform:String,video_id:String}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Event {
    schema_version: String, event_id: String, source: String,
    occurred_at_ms: i64, kind: String, payload: Value,
    #[serde(default)] profile_id: Option<String>,
    #[serde(skip)] extension_id: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PauseCommand { paused: bool }
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RuleInput { name: String, domain: String, path_prefix: Option<String>, minutes: Option<u32>, budget_minutes: Option<u32>, #[serde(default)] config: policy::RuleConfig }
#[derive(Serialize)]
struct RuleOutput { id: String, name: String, domain: String, path_prefix: Option<String>, expires_at_ms: Option<i64>, budget_minutes: Option<u32>, enabled: bool, enforced: bool, config: policy::RuleConfig, effective_until_ms: Option<i64> }

fn err(code: StatusCode, msg: &str) -> (StatusCode, String) { (code, msg.to_owned()) }
fn now_ms() -> i64 { Utc::now().timestamp_millis() }
fn data_dir() -> std::io::Result<PathBuf> {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(|| {
        std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."))
    });
    Ok(base.join("DigitalDiscipline"))
}
fn token_path() -> std::io::Result<PathBuf> { Ok(data_dir()?.join("agent.token")) }
pub(crate) fn read_token() -> std::io::Result<String> { Ok(fs::read_to_string(token_path()?)?.trim().to_owned()) }
fn ensure_token() -> std::io::Result<String> {
    let dir = data_dir()?; fs::create_dir_all(&dir)?;
    if let Ok(value) = read_token() { if value.len() == 64 { return Ok(value) } }
    let mut bytes = [0u8; 32]; rand::thread_rng().fill_bytes(&mut bytes);
    let value = hex::encode(bytes);
    // Only the per-user agent process creates the secret. Never print or expose it to the GUI.
    fs::write(token_path()?, &value)?;
    Ok(value)
}
fn init_db() -> Result<Connection, Box<dyn std::error::Error>> {
    let path = data_dir()?.join("discipline.sqlite3");
    let db = Connection::open(path)?;
    db.busy_timeout(StdDuration::from_secs(3))?;
    initialize_schema(&db)?;
    Ok(db)
}

fn initialize_schema(db:&Connection)->rusqlite::Result<()>{
    if db.query_row("PRAGMA user_version",[],|r|r.get::<_,i32>(0))?>14{return Err(rusqlite::Error::InvalidQuery);}
    db.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS sessions (id TEXT PRIMARY KEY, start_ms INTEGER NOT NULL, end_ms INTEGER NOT NULL, app TEXT NOT NULL, idle INTEGER NOT NULL, domain TEXT);
        CREATE INDEX IF NOT EXISTS idx_sessions_start ON sessions(start_ms);
        CREATE TABLE IF NOT EXISTS raw_events (event_id TEXT PRIMARY KEY, kind TEXT NOT NULL, occurred_ms INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS videos (instance_id TEXT PRIMARY KEY, platform TEXT NOT NULL, video_id TEXT NOT NULL, watched_ms INTEGER NOT NULL DEFAULT 0, qualified INTEGER NOT NULL DEFAULT 0, occurred_ms INTEGER NOT NULL);
        CREATE INDEX IF NOT EXISTS idx_videos_time ON videos(occurred_ms);
        CREATE TABLE IF NOT EXISTS rules (id TEXT PRIMARY KEY, name TEXT NOT NULL, domain TEXT NOT NULL, path_prefix TEXT, expires_ms INTEGER, budget_minutes INTEGER, enabled INTEGER NOT NULL, created_ms INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS settings (key TEXT PRIMARY KEY, value TEXT NOT NULL);
        INSERT OR IGNORE INTO settings(key,value) VALUES('paused','false');
        ")?;
    features::migrate(db)?;
    workflow::migrate(db)?;
    devices::migrate(db)?;
    bridge::migrate(db)?;
    schema::migrate(db)?;
    Ok(())
}

fn observe_tick(state: &AppState, mut sample: Observation) {
    if state.paused.load(Ordering::SeqCst)||state.stopping.load(Ordering::SeqCst) { return; }
    if let Ok(Some((domain,profile)))=tracking::foreground(&state.db.lock().unwrap(),&sample.app,sample.at_ms){
        sample.domain=domain;sample.profile_id=Some(profile);
    }
    // Keep privacy exclusions ahead of storage; skipping closes the current observation.
    {
        let db=state.db.lock().unwrap();
        if data_lifecycle::is_private(&db,sample.at_ms)||features::excluded(&db,&sample.app,sample.domain.as_deref()) {drop(db);stop_observing(state);return;}
    }
    let mut tracker = state.tracker.lock().unwrap();
    if state.stopping.load(Ordering::SeqCst) { return; }
    let db = state.db.lock().unwrap();
    if state.paused.load(Ordering::SeqCst)||data_lifecycle::is_private(&db,now_ms())||features::excluded(&db,&sample.app,sample.domain.as_deref()){tracker.reconciler.stop();tracker.session_id=None;return;}
    let (ended, current) = tracker.reconciler.push(sample);
    let mut failed=false;
    if let Some(previous) = ended {
        if let Some(id) = tracker.session_id.take() {
            if db.execute("UPDATE sessions SET end_ms=?1 WHERE id=?2",params![previous.end_ms,id]).is_err(){failed=true;}
        }
    }
    if let Some(s) = current {
        if !failed {
            if let Some(id) = &tracker.session_id {
                if db.execute("UPDATE sessions SET end_ms=?1 WHERE id=?2",params![s.end_ms,id]).is_err(){failed=true;}
            } else {
                let id=Uuid::new_v4().to_string();
                match db.execute("INSERT INTO sessions(id,start_ms,end_ms,app,idle,domain,profile_id) VALUES(?1,?2,?3,?4,?5,?6,?7)",
                     params![id,s.start_ms,s.end_ms,s.app,s.idle as u8,s.domain,s.profile_id]){
                    Ok(1)=>tracker.session_id=Some(id), _=>failed=true
                }
            }
        }
    }
    if failed {
        // Drop any unpersisted in-memory interval instead of backfilling a
        // fabricated gap when the database becomes writable again.
        tracker.session_id=None;
        tracker.reconciler.stop();
        record_storage_failure(state);
    }
}
fn record_storage_failure(state:&AppState){
    state.storage_last_error_ms.store(now_ms(),Ordering::SeqCst);
    state.storage_error_count.fetch_add(1,Ordering::SeqCst);
}
fn stop_observing(state: &AppState) {
    let mut tracker = state.tracker.lock().unwrap();
    let end = tracker.reconciler.stop();
    if let (Some(segment),Some(id))=(end,tracker.session_id.take()){
        if state.db.lock().unwrap().execute("UPDATE sessions SET end_ms=?1 WHERE id=?2",params![segment.end_ms,id]).is_err(){
            record_storage_failure(state);
        }
    }
}
async fn auth(State(state): State<Arc<AppState>>, request: Request, next: Next) -> Response {
    if request.headers().get("host").and_then(|v|v.to_str().ok())!=Some("127.0.0.1:48761") || request.headers().contains_key("origin") {
        return (StatusCode::FORBIDDEN,"desktop_loopback_only").into_response();
    }
    let presented = request.headers().get(AUTHORIZATION).and_then(|v| v.to_str().ok());
    let expected = format!("Bearer {}", state.token);
    if presented != Some(expected.as_str()) { return StatusCode::UNAUTHORIZED.into_response(); }
    next.run(request).await
}

async fn health(State(state):State<Arc<AppState>>) -> ApiResult {
    let version:i32=state.db.lock().unwrap().query_row("PRAGMA user_version",[],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"application":"digital-discipline-agent","version":env!("CARGO_PKG_VERSION"),"schema_version":domain::SCHEMA_VERSION,"storage_version":version,"tracking_paused":state.paused.load(Ordering::SeqCst),"private_session":data_lifecycle::private_info(&state.db.lock().unwrap(),now_ms()),"stopping":state.stopping.load(Ordering::SeqCst)})))
}
async fn shutdown_agent(State(state):State<Arc<AppState>>) -> Json<Value> {
    state.stopping.store(true,Ordering::SeqCst);
    stop_observing(&state);
    state.shutdown.notify_one();
    Json(json!({"ok":true,"shutting_down":true}))
}
async fn pause(State(state): State<Arc<AppState>>, Json(cmd): Json<PauseCommand>) -> ApiResult {
    state.paused.store(cmd.paused, Ordering::SeqCst);
    if cmd.paused { stop_observing(&state); }
    state.db.lock().unwrap().execute("INSERT OR REPLACE INTO settings(key,value) VALUES('paused',?1)", params![cmd.paused.to_string()]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"paused":cmd.paused,"blocking_unchanged":true})))
}

fn event_receipt(event_id:&str,disposition:&str)->Json<Value> {
    Json(json!({"ok":true,"event_id":event_id,"disposition":disposition,
        "duplicate":disposition=="duplicate","ignored_due_to_pause":disposition=="ignored_paused",
        "ignored_due_to_exclusion":disposition=="ignored_excluded"}))
}
async fn event_ingest(State(state): State<Arc<AppState>>, Json(mut event): Json<Event>) -> ApiResult {
    if state.stopping.load(Ordering::SeqCst){return Err(err(StatusCode::SERVICE_UNAVAILABLE,"agent_stopping_retry"));}
    if event.schema_version != domain::SCHEMA_VERSION || event.source != "extension" ||
        event.event_id.len() > 80 || event.event_id.len() < 8 ||
        event.profile_id.as_ref().is_some_and(|p|!tracking::valid_profile(p)) ||
        event.occurred_at_ms > now_ms() + 300_000 || event.occurred_at_ms < now_ms() - 86_400_000 {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_envelope"));
    }
    enum Parsed { Tab(BrowserPayload), Video(VideoPayload), Visit(VisitPayload), Health, Ack(BlockAck) }
    let parsed = match event.kind.as_str() {
        "browser.active_tab" | "browser.window_blurred" => {
            let p: BrowserPayload = serde_json::from_value(event.payload).map_err(|_|err(StatusCode::BAD_REQUEST,"invalid_browser_payload"))?;
            if !["brave","chrome","edge"].contains(&p.browser.as_str()) ||
                p.domain.as_ref().is_some_and(|d| !domain::valid_domain(d)) ||
                p.profile_id.as_ref().is_some_and(|x|Uuid::parse_str(x).is_err()) {
                return Err(err(StatusCode::BAD_REQUEST,"invalid_browser_domain"));
            }
            Parsed::Tab(p)
        }
        "video.qualified_play" | "video.progress" => {
            let p: VideoPayload = serde_json::from_value(event.payload).map_err(|_|err(StatusCode::BAD_REQUEST,"invalid_video_payload"))?;
            if !["youtube_shorts", "youtube_watch", "instagram_reels"].contains(&p.platform.as_str()) ||
                p.video_id.is_empty() || p.video_id.len()>128 || !p.video_id.bytes().all(|c| c.is_ascii_alphanumeric() || c==b'-' || c==b'_') ||
                p.instance_id.len()>80 || p.instance_id.len()<8 || p.watched_ms<0 || p.watched_ms>86_400_000 ||
                (event.kind == "video.qualified_play" && p.watched_ms < domain::PLAY_QUALIFY_MS) {
                return Err(err(StatusCode::BAD_REQUEST,"invalid_video_data"));
            }
            Parsed::Video(p)
        }
        "video.navigation" => {
            let p:VisitPayload=serde_json::from_value(event.payload).map_err(|_|err(StatusCode::BAD_REQUEST,"invalid_video_payload"))?;
            if !["youtube_shorts","youtube_watch","instagram_reels"].contains(&p.platform.as_str())||p.video_id.is_empty()||p.video_id.len()>128||!p.video_id.bytes().all(|c|c.is_ascii_alphanumeric()||b"-_".contains(&c)){return Err(err(StatusCode::BAD_REQUEST,"invalid_video_data"));}
            Parsed::Visit(p)
        },
        "browser.health" => Parsed::Health,
        "block.ack" => {
            let p:BlockAck=serde_json::from_value(event.payload).map_err(|_|err(StatusCode::BAD_REQUEST,"invalid_block_ack"))?;
            if p.applied_count>500 {return Err(err(StatusCode::BAD_REQUEST,"invalid_rule_count"));}
            Parsed::Ack(p)
        },
        _ => return Err(err(StatusCode::BAD_REQUEST,"unsupported_kind")),
    };
    if let Parsed::Tab(p)=&parsed{
        if event.profile_id.is_some()&&p.profile_id.is_some()&&event.profile_id!=p.profile_id{return Err(err(StatusCode::BAD_REQUEST,"profile_identity_mismatch"));}
        if event.profile_id.is_none(){event.profile_id=p.profile_id.clone();}
    }
    if state.paused.load(Ordering::SeqCst) && event.kind!="block.ack" {
        return Ok(event_receipt(&event.event_id,"ignored_paused"));
    }
    // Commit the raw event ID and its derived video update atomically.
    // A retry after a failed derived write must never be discarded as a duplicate.
    let mut db = state.db.lock().unwrap();
    let tx = db.transaction().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    if event.kind!="block.ack" {
        let subject=match &parsed{Parsed::Tab(p)=>p.domain.as_ref().map(|d|format!("domain:{d}")),Parsed::Video(p)=>Some(format!("domain:{}",if p.platform.starts_with("youtube_"){"www.youtube.com"}else{"www.instagram.com"})),Parsed::Visit(p)=>Some(format!("domain:{}",if p.platform.starts_with("youtube_"){"www.youtube.com"}else{"www.instagram.com"})),_=>None};
        if let Some(reason)=data_lifecycle::rejection(&tx,event.occurred_at_ms,subject.as_deref()).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?{return Ok(event_receipt(&event.event_id,reason));}
    }
    let inserted = tx.execute("INSERT OR IGNORE INTO raw_events(event_id,kind,occurred_ms,profile_id,extension_id) VALUES(?1,?2,?3,?4,?5)", params![event.event_id,event.kind,event.occurred_at_ms,event.profile_id,event.extension_id]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    if inserted == 0 {
        let identity=tx.query_row("SELECT profile_id,extension_id FROM raw_events WHERE event_id=?1",params![event.event_id],|r|Ok((r.get::<_,Option<String>>(0)?,r.get::<_,Option<String>>(1)?))).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
        if identity.0.as_ref().zip(event.profile_id.as_ref()).is_some_and(|(a,b)|a!=b)||identity.1.as_ref().zip(event.extension_id.as_ref()).is_some_and(|(a,b)|a!=b){return Err(err(StatusCode::UNPROCESSABLE_ENTITY,"event_identity_conflict"));}
        return Ok(event_receipt(&event.event_id,"duplicate"));
    }
    let mut pending_browser: Option<BrowserPayload> = None;
    let mut pending_ack: Option<BlockAck> = None;
    match parsed {
        Parsed::Tab(p) => pending_browser = Some(p),
        Parsed::Video(p) => {
            let host=if p.platform=="youtube_shorts" || p.platform=="youtube_watch"{"www.youtube.com"}else{"www.instagram.com"};
            if features::excluded(&tx,"",Some(host)){return Ok(event_receipt(&event.event_id,"ignored_excluded"));}
            let identity=tx.query_row("SELECT platform,video_id,profile_id,extension_id FROM videos WHERE instance_id=?1",params![p.instance_id],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,Option<String>>(2)?,r.get::<_,Option<String>>(3)?))).optional().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
            if identity.is_some_and(|(platform,id,profile,ext)|platform!=p.platform||id!=p.video_id||profile.as_ref().zip(event.profile_id.as_ref()).is_some_and(|(a,b)|a!=b)||ext.as_ref().zip(event.extension_id.as_ref()).is_some_and(|(a,b)|a!=b)){return Err(err(StatusCode::UNPROCESSABLE_ENTITY,"playback_identity_conflict"));}
            tx.execute("INSERT INTO videos(instance_id,platform,video_id,watched_ms,qualified,occurred_ms,profile_id,extension_id) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)
                ON CONFLICT(instance_id) DO UPDATE SET watched_ms=max(watched_ms,excluded.watched_ms), qualified=max(qualified,excluded.qualified),profile_id=COALESCE(videos.profile_id,excluded.profile_id),extension_id=COALESCE(videos.extension_id,excluded.extension_id)",
                params![p.instance_id,p.platform,p.video_id,p.watched_ms, (event.kind == "video.qualified_play") as u8,event.occurred_at_ms,event.profile_id,event.extension_id])
                .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
        }
        Parsed::Visit(p)=>{
            let host=if p.platform.starts_with("youtube_"){"www.youtube.com"}else{"www.instagram.com"};
            if features::excluded(&tx,"",Some(host)){return Ok(event_receipt(&event.event_id,"ignored_excluded"));}
            tx.execute("INSERT INTO video_visits(event_id,platform,video_id,occurred_ms,profile_id,extension_id) VALUES(?1,?2,?3,?4,?5,?6)",params![event.event_id,p.platform,p.video_id,event.occurred_at_ms,event.profile_id,event.extension_id]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
        },
        Parsed::Ack(p) => {pending_ack=Some(p);},
        Parsed::Health => {}
    }
    tracking::record_event(&tx,event.extension_id.as_deref(),event.profile_id.as_deref(),pending_browser.as_ref().map(|b|b.browser.as_str()),pending_browser.as_ref().and_then(|b|b.domain.as_deref()),pending_browser.as_ref().is_some_and(|b|b.focused),event.occurred_at_ms,event.kind=="video.qualified_play",event.kind=="block.ack").map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    tx.commit().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    drop(db);
    if let Some(p) = pending_browser {
        // Browser events only annotate native foreground sessions, never add separate time.
        let mut b = state.browser.lock().unwrap();
        b.domain=p.domain;b.profile_id=p.profile_id;b.browser=p.browser;b.focused=p.focused;b.last_event_ms=event.occurred_at_ms;
    }
    if let Some(ack)=pending_ack {
        let mut b=state.browser.lock().unwrap();b.rule_sync_ms=event.occurred_at_ms;b.rule_count=ack.applied_count;
    }
    Ok(event_receipt(&event.event_id,"accepted"))
}
fn midnight_local_ms(day: NaiveDate) -> i64 {
    let local = Local.from_local_datetime(&day.and_hms_opt(0,0,0).unwrap());
    local.earliest().or_else(|| local.latest()).map(|x| x.timestamp_millis()).unwrap_or(0)
}
fn get_rules(db: &Connection, now: i64) -> rusqlite::Result<Vec<RuleOutput>> {
    let mut stmt=db.prepare("SELECT id,name,domain,path_prefix,expires_ms,budget_minutes,enabled,config_json FROM rules ORDER BY created_ms DESC")?;
    let rows=stmt.query_map([],|row|{
        let config:String=row.get(7)?;
        Ok(RuleOutput{id:row.get(0)?,name:row.get(1)?,domain:row.get(2)?,path_prefix:row.get(3)?,
            expires_at_ms:row.get(4)?,budget_minutes:row.get(5)?,enabled:row.get::<_,i64>(6)?!=0,
            enforced:false,config:serde_json::from_str(&config).unwrap_or_default(),effective_until_ms:None})
    })?;
    let mut rules=rows.collect::<rusqlite::Result<Vec<_>>>()?;
    let today=midnight_local_ms(Local::now().date_naive());
    let tomorrow=midnight_local_ms(Local::now().date_naive()+Duration::days(1));
    for rule in &mut rules {
        if !rule.enabled || rule.expires_at_ms.is_some_and(|expiry|expiry<=now) || !policy::schedule_active(&rule.config,Local::now()) {continue;}
        let mut domains=vec![rule.domain.clone()];domains.extend(rule.config.group_domains.iter().cloned());
        let mut matching=true;
        if let Some(limit)=rule.budget_minutes {matching=analytics::active_target_ms(db,&domains,&rule.config.group_apps,&rule.config.group_categories,
            &rule.config.excluded_domains,&rule.config.excluded_apps,today,now)?>=limit as i64*60_000;}
        if let Some(limit)=rule.config.qualified_plays {matching=matching && analytics::qualified_count(db,&domains,today,rule.config.include_replays)?>=limit as i64;}
        if let Some(limit)=rule.config.streak_plays {
            let window=rule.config.streak_window_minutes.unwrap_or(15) as i64*60_000;
            matching=matching && analytics::rolling_play_count(db,&domains,now-window,
                rule.config.streak_gap_seconds.unwrap_or(120) as i64*1000)?>=limit as i64;
        }
        rule.enforced=matching;
        // A short fail-open browser lease prevents stale policies if agent is lost.
        // Daily budgets reset at local midnight; recurring schedules at their wall-clock boundaries.
        if matching {
            let mut bound=now+180_000;
            if let Some(end)=rule.expires_at_ms {bound=bound.min(end);}
            if rule.budget_minutes.is_some() || rule.config.qualified_plays.is_some(){bound=bound.min(tomorrow);}
            if let Some(b)=policy::next_schedule_boundary(&rule.config,Local::now()){bound=bound.min(b);}
            rule.effective_until_ms=Some(bound);
        }
    }
    Ok(rules)
}
async fn rules(State(state): State<Arc<AppState>>) -> ApiResult {
    let db=state.db.lock().unwrap();let r=get_rules(&db,now_ms()).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"rules":r})))
}
async fn extension_rules(State(state): State<Arc<AppState>>) -> ApiResult {extension_rules_for_profile(state,None).await}
async fn extension_rules_for_profile(state:Arc<AppState>,profile:Option<String>)->ApiResult{
    let mut db=state.db.lock().unwrap();let now=now_ms();let r=get_rules(&db,now).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    // Durable transitions only, no per-minute log spam; policies remain deterministic.
    if let Ok(tx)=db.transaction() {
        for rule in &r {
            let existing=tx.query_row("SELECT active FROM rule_states WHERE rule_id=?1",params![rule.id],|r|r.get::<_,i64>(0)).ok();
            let active=rule.enforced as i64;
            if existing!=Some(active) {
                let _=tx.execute("INSERT INTO rule_states(rule_id,active,changed_ms) VALUES(?1,?2,?3) ON CONFLICT(rule_id) DO UPDATE SET active=excluded.active,changed_ms=excluded.changed_ms",params![rule.id,active,now]);
                let _=tx.execute("INSERT INTO audits(kind,detail,occurred_ms) VALUES('rule_transition',?1,?2)",params![json!({"rule_id":rule.id,"rule_name":rule.name,"triggered":rule.enforced}).to_string(),now]);
            }
        }
        let _=tx.commit();
    }
    let mut blocks=r.iter().filter(|r|r.enforced).map(|r|json!({"id":r.id,"domain":r.domain,"path_prefix":r.path_prefix,
        "action":r.config.action(),"reason":r.name,"expires_at_ms":r.effective_until_ms})).collect::<Vec<_>>();
    if let Ok(Some(f))=features::active_focus(&db,now) {
        let end=f["end_ms"].as_i64().unwrap_or(now);
        let original=f["difficulty"].as_str().unwrap_or("Gentle");
        let selected=focus_workflow::restrictions(&db,f["id"].as_str().unwrap_or(""),original).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"focus_workflow_storage_failed"))?;
        let difficulty=selected.as_deref().unwrap_or("Gentle");
        if difficulty!="Gentle" {
            if let Some(domains)=f["block_domains"].as_array(){
                for d in domains.iter().filter_map(Value::as_str).take(15){
                    if domain::valid_domain(d){
                        // Gentle = informational timer; Balanced = in-page feed reduction
                        // on known adapters / optional reminder elsewhere; Strict = navigation block.
                        let action=if difficulty=="Strict" {"block"}
                            else if ["www.youtube.com","www.instagram.com"].contains(&d) {"hide_feed"}
                            else {"notify"};
                        blocks.push(json!({"id":f["id"],"domain":d,"path_prefix":null,"action":action,
                            "reason":format!("{} focus",difficulty),"expires_at_ms":end.min(now+180_000)}));
                    }
                }
            }
        }
    }
    let (blocks,next_policy_check)=policy_exceptions::apply(&db,blocks,profile.as_deref(),now).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"policy_exception_storage_failed"))?;
    Ok(Json(json!({"ok":true,"schema_version":domain::SCHEMA_VERSION,"blocks":blocks,"checked_at_ms":now,"lease_ms":180000,"presentation_private":features::read_setting(&db,"presentation_privacy")=="true","purge_before_ms":features::read_setting(&db,"activity_deleted_before_ms").parse::<i64>().unwrap_or(0),"private_until_ms":features::read_setting(&db,"private_until_ms").parse::<i64>().unwrap_or(0),"next_policy_check_ms":next_policy_check})))
}
async fn preview_rules(State(state):State<Arc<AppState>>,Json(cmd):Json<RuleInput>)->ApiResult {
    let domain=cmd.domain.trim().to_ascii_lowercase();
    if !domain::valid_domain(&domain) || !cmd.config.validate() ||
        cmd.path_prefix.as_ref().is_some_and(|x|!domain::safe_path(x)) ||
        cmd.budget_minutes.is_some_and(|x|x<1||x>1440) ||
        cmd.config.qualified_plays.is_some_and(|x|x<1||x>10000) ||
        cmd.config.streak_plays.is_some_and(|x|x<2||x>500) ||
        cmd.minutes.is_some_and(|x|x==0||x>1440) ||
        ([cmd.minutes.is_some(),cmd.budget_minutes.is_some(),cmd.config.qualified_plays.is_some(),cmd.config.streak_plays.is_some()].iter().filter(|x|**x).count()>1) {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_preview_rule"));
    }
    let now=now_ms();let mut domains=vec![domain.clone()];domains.extend(cmd.config.group_domains.clone());
    let db=state.db.lock().unwrap();let start=midnight_local_ms(Local::now().date_naive());
    let minutes=analytics::active_target_ms(&db,&domains,&cmd.config.group_apps,&cmd.config.group_categories,
        &cmd.config.excluded_domains,&cmd.config.excluded_apps,start,now).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let plays=analytics::qualified_count(&db,&domains,start,cmd.config.include_replays).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let rolling=if cmd.config.streak_plays.is_some(){analytics::rolling_play_count(&db,&domains,now-cmd.config.streak_window_minutes.unwrap_or(15) as i64*60_000,
        cmd.config.streak_gap_seconds.unwrap_or(120) as i64*1000).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?}else{0};
    let in_schedule=policy::schedule_active(&cmd.config,Local::now());
    let matched=in_schedule && cmd.budget_minutes.is_none_or(|x|minutes>=x as i64*60_000) &&
        cmd.config.qualified_plays.is_none_or(|x|plays>=x as i64) && cmd.config.streak_plays.is_none_or(|x|rolling>=x as i64);
    Ok(Json(json!({"ok":true,"would_activate_now":matched,"schedule_active":in_schedule,
        "observed_eligible_minutes_today":minutes as f64/60_000.0,"qualified_plays_today":plays,"rolling_streak":rolling,
        "signal":"Preview uses only available local observed data. Creating a rule does not alter past activity. Browser enforcement still requires a paired extension and consent."})))
}

async fn rule_audit(State(state):State<Arc<AppState>>)->ApiResult {
    let db=state.db.lock().unwrap();
    let mut q=db.prepare("SELECT detail,occurred_ms FROM audits WHERE kind='rule_transition' ORDER BY occurred_ms DESC LIMIT 120")
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let rows=q.query_map([],|r|Ok(json!({"detail":serde_json::from_str::<Value>(&r.get::<_,String>(0)?).unwrap_or(Value::Null),"occurred_ms":r.get::<_,i64>(1)?})))
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?.collect::<rusqlite::Result<Vec<_>>>()
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"events":rows,"note":"Engine transition decisions; browser delivery is separately acknowledged."})))
}

async fn create_rule(State(state): State<Arc<AppState>>, Json(cmd): Json<RuleInput>) -> ApiResult {
    let d=cmd.domain.trim().to_ascii_lowercase();
    if !domain::valid_domain(&d) || cmd.name.is_empty() || cmd.name.len()>80 ||
        cmd.path_prefix.as_ref().is_some_and(|p|!domain::safe_path(p)) ||
        (!cmd.config.validate()) ||
        (["hide_feed","stop_autoplay"].contains(&cmd.config.action()) && !["www.youtube.com","www.instagram.com"].contains(&d.as_str())) ||
        // Allow independent indefinite, timed, daily minutes, and qualifying-play budgets.
        (cmd.minutes.is_some() && (cmd.budget_minutes.is_some() || cmd.config.qualified_plays.is_some() || cmd.config.streak_plays.is_some())) ||
        (cmd.budget_minutes.is_some() && (cmd.config.qualified_plays.is_some() || cmd.config.streak_plays.is_some())) ||
        (cmd.config.qualified_plays.is_some() && cmd.config.streak_plays.is_some()) ||
        cmd.minutes.is_some_and(|m| m<1 || m>1440) || cmd.budget_minutes.is_some_and(|m|m<1 || m>1440) {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_rule"));
    }
    let id=Uuid::new_v4().to_string(); let expires=cmd.minutes.map(|m|now_ms()+m as i64 *60_000);
    state.db.lock().unwrap().execute("INSERT INTO rules(id,name,domain,path_prefix,expires_ms,budget_minutes,enabled,created_ms,config_json) VALUES(?1,?2,?3,?4,?5,?6,1,?7,?8)",
        params![id,cmd.name,d,cmd.path_prefix,expires,cmd.budget_minutes,now_ms(),serde_json::to_string(&cmd.config).unwrap()]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"id":id,"expires_at_ms":expires,"enforcement":"extension_sync_pending"})))
}
async fn remove_rule(Path(id): Path<String>,State(state): State<Arc<AppState>>) -> ApiResult {
    if Uuid::parse_str(&id).is_err() { return Err(err(StatusCode::BAD_REQUEST,"invalid_rule_id")); }
    let n=state.db.lock().unwrap().execute("UPDATE rules SET enabled=0 WHERE id=?1",params![id]).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"disabled":n>0,"extension_sync_pending":true})))
}

#[derive(Serialize)]
struct SessionDto { id:String, start_ms:i64,end_ms:i64,app:String,idle:bool,domain:Option<String>,profile_id:Option<String>,category:String }
fn get_sessions(db: &Connection, earliest: i64, latest: i64, cap: usize) -> rusqlite::Result<Vec<SessionDto>> {
    let overrides=features::load_categories(db)?;
    let mut q=db.prepare("SELECT id,start_ms,end_ms,app,idle,domain,profile_id FROM sessions WHERE end_ms>=?1 AND start_ms<=?2 ORDER BY start_ms DESC LIMIT ?3")?;
    let items=q.query_map(params![earliest,latest,cap as i64],|r|{
        let app:String=r.get(3)?;let domain:Option<String>=r.get(5)?;
        Ok(SessionDto {id:r.get(0)?,start_ms:r.get(1)?,end_ms:r.get(2)?,category:features::resolved_category(&overrides,&app,domain.as_deref()),app,idle:r.get::<_,i64>(4)?!=0,domain,profile_id:r.get(6)?})
    })?.collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(items)
}

async fn snapshot_inner(state:Arc<AppState>,compact:bool) -> ApiResult {
    let now=now_ms(); let today=Local::now().date_naive();let start=midnight_local_ms(today);
    let db=state.db.lock().unwrap();
    let recent=get_sessions(&db,start,now,if compact{5}else{180}).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let all=get_sessions(&db,start,now,50000).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let active=policy::union_ms(all.iter().filter(|s|!s.idle).map(|s|(s.start_ms,s.end_ms)).collect(),start,now);
    let development=policy::union_ms(all.iter().filter(|s|!s.idle&&s.category=="Development").map(|s|(s.start_ms,s.end_ms)).collect(),start,now);
    let entertainment=policy::union_ms(all.iter().filter(|s|!s.idle&&s.category=="Entertainment").map(|s|(s.start_ms,s.end_ms)).collect(),start,now);
    let heatmap:Vec<Value>=Vec::new();
    // "Distraction" exists only when the owner explicitly chooses categories.
    // Never assert an activity is wasted based on our curated taxonomy.
    let distraction_labels:Vec<String>=serde_json::from_str(&features::read_setting(&db,"distraction_categories")).unwrap_or_default();
    let distraction=if distraction_labels.is_empty(){None}else{
        Some(policy::union_ms(all.iter().filter(|s|!s.idle && distraction_labels.contains(&s.category))
            .map(|s|(s.start_ms,s.end_ms)).collect(),start,now))
    };
    let plays:i64=db.query_row("SELECT COUNT(*) FROM videos WHERE platform!='youtube_watch' AND qualified=1 AND occurred_ms>=?1",params![start],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"metrics_storage_failed"))?;
    let uniques:i64=db.query_row("SELECT COUNT(DISTINCT platform || ':' || video_id) FROM videos WHERE platform!='youtube_watch' AND qualified=1 AND occurred_ms>=?1",params![start],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"metrics_storage_failed"))?;
    let playback:i64=db.query_row("SELECT COALESCE(SUM(watched_ms),0) FROM videos WHERE platform!='youtube_watch' AND occurred_ms>=?1",params![start],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"metrics_storage_failed"))?;
    let watch_plays:i64=db.query_row("SELECT COUNT(*) FROM videos WHERE platform='youtube_watch' AND qualified=1 AND occurred_ms>=?1",params![start],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"metrics_storage_failed"))?;
    let watch_uniques:i64=db.query_row("SELECT COUNT(DISTINCT video_id) FROM videos WHERE platform='youtube_watch' AND qualified=1 AND occurred_ms>=?1",params![start],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"metrics_storage_failed"))?;
    let watch_playback:i64=db.query_row("SELECT COALESCE(SUM(watched_ms),0) FROM videos WHERE platform='youtube_watch' AND occurred_ms>=?1",params![start],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"metrics_storage_failed"))?;
    let route_visits:i64=db.query_row("SELECT COUNT(*) FROM video_visits WHERE occurred_ms>=?1",params![start],|r|r.get(0)).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"metrics_storage_failed"))?;
    let rules=if compact{Vec::new()}else{get_rules(&db,now).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?};
    let current=recent.first().filter(|s|now-s.end_ms<8_000);
    let focus=features::active_focus(&db,now).ok().flatten();
    let focus_hint=focus.as_ref().and_then(|f|features::focus_goal_hint(&db,f,current,now));
    Ok(Json(json!({
        "ok":true,"schema_version":domain::SCHEMA_VERSION,"observed_at_ms":now,
        "private_session":data_lifecycle::private_info(&db,now),"tracking_paused":state.paused.load(Ordering::SeqCst),"platform_supported":cfg!(windows),
        "browser_connected":db.query_row("SELECT EXISTS(SELECT 1 FROM browser_instances WHERE revoked=0 AND last_seen_ms>?1)",params![now-90000],|r|r.get::<_,bool>(0)).unwrap_or(false),
        "browser_rules_acknowledged":state.browser.lock().unwrap().rule_sync_ms>now-180_000,
        "browser_rules_count":state.browser.lock().unwrap().rule_count,
        "metrics":{"route_visits":route_visits,"active_ms":active,"development_ms":development,"entertainment_ms":entertainment,"distraction_ms":distraction,"qualified_plays":plays,"unique_videos":uniques,"replays":(plays-uniques).max(0),"playback_ms":playback,"watch_qualified_plays":watch_plays,"watch_unique_videos":watch_uniques,"watch_playback_ms":watch_playback},
        "current":current,"sessions":recent,"heatmap":heatmap,"rules":rules,
        "goals":if compact{Vec::<Value>::new()}else{features::read_goals(&db).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?},
        "focus":focus,"focus_goal_hint":focus_hint,
        "profile_receipts":tracking::receipts(&db).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?,
        "coverage":{"windows_collector":if !cfg!(windows){"unsupported"}else if state.storage_last_error_ms.load(Ordering::SeqCst)>now-120_000{"degraded_storage"}else{"running"},"browser":"best_effort","rule_enforcement":"not_acknowledged_per_profile","no_data_is_not_zero":true,"today_may_be_truncated":all.len()==50000,"last_native_observation_ms":recent.first().map(|s|s.end_ms)}
    })))
}
async fn snapshot(State(state):State<Arc<AppState>>)->ApiResult{snapshot_inner(state,false).await}
async fn live(State(state):State<Arc<AppState>>)->ApiResult{snapshot_inner(state,true).await}
async fn day_sessions(Path(day): Path<String>, State(state): State<Arc<AppState>>) -> ApiResult {
    let date=NaiveDate::parse_from_str(&day,"%Y-%m-%d")
        .map_err(|_|err(StatusCode::BAD_REQUEST,"invalid_date"))?;
    let start=midnight_local_ms(date);
    let end=midnight_local_ms(date+Duration::days(1));
    let db=state.db.lock().unwrap();
    let sessions=get_sessions(&db,start,end,2500).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    Ok(Json(json!({"ok":true,"date":day,"sessions":sessions,"may_be_truncated":sessions.len()==2500})))
}

async fn export_data(State(state): State<Arc<AppState>>) -> ApiResult {
    let db=state.db.lock().unwrap();
    let sessions=get_sessions(&db,0,now_ms(),50_000).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let mut q=db.prepare("SELECT instance_id,platform,video_id,watched_ms,qualified,occurred_ms FROM videos ORDER BY occurred_ms DESC LIMIT 50000")
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let videos=q.query_map([],|row|Ok(json!({
        "instance_id":row.get::<_,String>(0)?,"platform":row.get::<_,String>(1)?,"video_id":row.get::<_,String>(2)?,
        "watched_ms":row.get::<_,i64>(3)?,"qualified":row.get::<_,i64>(4)?>0,"occurred_ms":row.get::<_,i64>(5)?
    }))).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?
        .collect::<rusqlite::Result<Vec<_>>>().map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    let truncated=sessions.len()==50_000 || videos.len()==50_000;
    Ok(Json(json!({"schema_version":"1.0","exported_at_ms":now_ms(),"sessions":sessions,"videos":videos,"truncated":truncated,"scope":"app_basenames_and_opt_in_domains_and_observed_video_ids"})))
}
async fn delete_data(State(state): State<Arc<AppState>>) -> ApiResult {
    let mut tracker=state.tracker.lock().unwrap();tracker.reconciler.stop();tracker.session_id=None;
    let db=state.db.lock().unwrap();data_lifecycle::clear_activity(&db,now_ms()).map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"storage_failed"))?;
    *state.browser.lock().unwrap()=BrowserStatus::default();
    Ok(Json(json!({"ok":true,"deleted":"activity_video_classifications_memory_and_imported_devices","rules_and_owner_overrides_retained":true,"queued_history_before_deletion_rejected":true})))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Chromium starts native hosts with the extension origin as the first argument;
    // a separate --native-host flag is supported for explicit test invocation.
    let native_invocation = std::env::args().skip(1).take(4).any(|arg|
        arg == "--native-host" || arg.starts_with("chrome-extension://"));
    if native_invocation {
        return native::run();
    }
    let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
    rt.block_on(run_agent())
}

async fn run_agent() -> Result<(),Box<dyn std::error::Error>> {
    // Claim the collector port before opening/migrating storage or spawning workers.
    // A second agent exits without changing an active collector's database/token.
    let listener=tokio::net::TcpListener::bind("127.0.0.1:48761").await?;
    fs::create_dir_all(data_dir()?)?;
    let token=ensure_token()?;
    let db=init_db()?;
    let paused=db.query_row("SELECT value FROM settings WHERE key='paused'",[],|r|r.get::<_,String>(0)).unwrap_or_default()=="true";
    let state=Arc::new(AppState {db:Mutex::new(db),tracker:Mutex::new(Tracker{reconciler:Reconciler::default(),session_id:None}),browser:Mutex::new(BrowserStatus::default()),paused:AtomicBool::new(paused),stopping:AtomicBool::new(false),storage_last_error_ms:AtomicI64::new(0),storage_error_count:AtomicU64::new(0),token,shutdown:Arc::new(tokio::sync::Notify::new())});
    let classify_state=Arc::clone(&state);
    let discovery_state=Arc::clone(&state);
    tokio::spawn(async move{loop{if discovery_state.stopping.load(Ordering::SeqCst){break;}service_discovery::tick(Arc::clone(&discovery_state)).await;let delay=if features::read_setting(&discovery_state.db.lock().unwrap(),"lite_mode")=="true"{12}else{3};tokio::time::sleep(StdDuration::from_secs(delay)).await;}});
    tokio::spawn(async move{
        loop {
            if classify_state.stopping.load(Ordering::SeqCst){break;}
            if let Ok(db)=classify_state.db.lock(){let _=classification::queue_unknowns(&db);}
            classification::worker(Arc::clone(&classify_state)).await;
            let delay=if features::read_setting(&classify_state.db.lock().unwrap(),"lite_mode")=="true"{180}else{75};
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        }
    });
    let prune_state=Arc::clone(&state);
    thread::spawn(move||loop {
        if prune_state.stopping.load(Ordering::SeqCst){break;}
        if let Ok(db)=prune_state.db.lock(){let _=features::prune(&db,now_ms());}
        thread::sleep(StdDuration::from_secs(3600));
    });
    let track=Arc::clone(&state);
    thread::spawn(move||loop {
        if track.stopping.load(Ordering::SeqCst){break;}
        let now=now_ms();
        if !track.paused.load(Ordering::SeqCst) {
            let threshold=track.db.lock().unwrap().query_row("SELECT value FROM settings WHERE key='idle_threshold_ms'",[],|r|r.get::<_,String>(0)).unwrap_or_default().parse::<u32>().unwrap_or(300_000);
            if let Some(sample)=windows::observe(now,threshold) { observe_tick(&track,sample); }
            else { stop_observing(&track); }
        }
        thread::sleep(StdDuration::from_secs(2));
    });
    let shutdown_signal=Arc::clone(&state.shutdown);
    let authenticated=Router::new()
        .route("/shutdown",post(shutdown_agent)).route("/health",get(health)).route("/state",get(snapshot)).route("/sessions/{day}",get(day_sessions)).route("/events",post(event_ingest))
        .route("/health/details",get(health::details)).route("/live",get(live)).route("/reports/heatmap",get(reporting::history))
        .route("/tracking/doctor",get(tracking::doctor)).route("/tracking/doctor/rendered",post(tracking::rendered)).route("/tracking/profiles/access",post(tracking::access))
        .route("/tracking/pause",post(pause)).route("/rules",get(rules).post(create_rule))
        .route("/rules/{id}",delete(remove_rule)).route("/extension/rules",get(extension_rules))
        .route("/policy/exceptions",get(policy_exceptions::list).post(policy_exceptions::create)).route("/policy/exceptions/{id}",delete(policy_exceptions::revoke)).route("/rules/preview",post(preview_rules)).route("/rules/audit",get(rule_audit))
        .route("/export",get(export_data)).route("/export/snapshot",post(export_snapshot::create)).route("/data",delete(delete_data)).route("/data/purge",post(data_lifecycle::purge)).route("/privacy/session",get(data_lifecycle::private_status).post(data_lifecycle::private_session)).route("/privacy/presentation",post(data_lifecycle::presentation))
        .route("/videos",get(features::video_history)).route("/insights",get(features::insights))
        .route("/categories",get(features::list_categories)).route("/categories/override",post(features::override_category))
        .route("/categories/override/{subject}",delete(features::delete_override))
        .route("/workflows",get(workflow::list))
        .route("/workflows/profiles",post(workflow::add_profile))
        .route("/workflows/profiles/{id}",delete(workflow::archive_profile).post(workflow::update_profile))
        .route("/workflows/tasks",post(workflow::add_task))
        .route("/workflows/tasks/{id}",post(workflow::transition_task))
        .route("/goals",get(features::goals).post(features::create_goal)).route("/goals/{id}",delete(features::delete_goal))
        .route("/inbox",get(inbox::list)).route("/inbox/{id}",post(inbox::dismiss))
        .route("/focus/cycles/{id}",post(focus_workflow::action)).route("/focus/reviews",get(focus_workflow::reviews)).route("/focus/reviews/{id}",post(focus_workflow::save_review)).route("/focus/start",post(features::start_focus)).route("/focus/end",post(features::end_focus)).route("/focus/hint/choice",post(features::choose_focus_hint))
        .route("/settings",get(features::get_settings).post(features::set_settings))
        .route("/coach",post(features::coach))
        .route("/coach/credentials",get(credentials::list).post(credentials::save))
        .route("/coach/credentials/{provider}",delete(credentials::remove))
        .route("/devices",get(devices::list)).route("/devices/export",get(devices::export))
        .route("/devices/import",post(devices::import)).route("/devices/{id}",delete(devices::forget))
        .route("/classifications",get(classification::list)).route("/classifications/config",post(classification::configure).delete(classification::remove_config))
        .route("/classifications/preferences",post(classification::preferences))
        .route("/companion",get(companion::status).post(companion::prepare))
        .route("/services",get(service_discovery::list)).route("/services/next",post(service_discovery::next)).route("/services/decision",post(service_discovery::decide)).route("/services/preferences",post(service_discovery::preferences))
        .route("/browser-pairs",get(bridge::list).post(bridge::enroll))
        .route("/browser-pairs/{id}",delete(bridge::revoke))
        .layer(middleware::from_fn_with_state(Arc::clone(&state),auth));
    let app=Router::new().merge(authenticated).merge(bridge::routes())
        .layer(DefaultBodyLimit::max(512_000)).with_state(state);
    eprintln!("Digital Discipline agent running on loopback (auth required)");
    axum::serve(listener,app).with_graceful_shutdown(async move {shutdown_signal.notified().await;}).await?;
    Ok(())
}

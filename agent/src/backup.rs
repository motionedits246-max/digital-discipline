//! Encrypted, versioned backups of owner data. All imports pass through known SQLite tables.
use crate::{err, features, now_ms, ApiResult, AppState};
use axum::{extract::State, http::StatusCode, Json};
use ring::{aead, pbkdf2, rand::{SecureRandom, SystemRandom}};
use rusqlite::{params, params_from_iter, types::{Value as SqlValue, ValueRef}, Connection};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    num::NonZeroU32,
    path::{Path, PathBuf},
    sync::{atomic::{AtomicBool, Ordering}, Arc},
};
use uuid::Uuid;
use zeroize::Zeroizing;

const MAGIC: &[u8] = b"DDBACKUP\x01";
const MAX_BYTES: usize = 64 * 1024 * 1024;
const ITERATIONS: u32 = 350_000;
const SALT_BYTES: usize = 16;
const NONCE_BYTES: usize = 12;

const CONFIG: &[&str] = &[
    "workflow_profiles", "workflow_tasks", "goals", "rules", "category_overrides",
    "planner_blocks", "weekly_targets", "focus_presets",
];
const HISTORY: &[&str] = &[
    "sessions", "videos", "video_visits", "raw_events", "focus_sessions", "focus_cycles",
    "focus_cycle_transitions", "focus_reviews", "history_corrections", "manual_activity",
    "reading_windows", "reading_segments", "reward_days", "reward_ledger",
];
const SETTINGS: &[&str] = &[
    "lite_mode", "retention_days", "idle_threshold_ms", "excluded_apps", "excluded_domains",
    "video_history_visible", "custom_categories", "distraction_categories", "shared_quiet_hours",
];

static BUSY: AtomicBool = AtomicBool::new(false);
struct Lease;
impl Lease {
    fn take() -> Result<Self, &'static str> {
        BUSY.compare_exchange(false,true,Ordering::SeqCst,Ordering::SeqCst)
            .map(|_|Self).map_err(|_|"backup_already_running")
    }
}
impl Drop for Lease {
    fn drop(&mut self) { BUSY.store(false,Ordering::SeqCst); }
}

#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    storage_version:i32,
    created_ms:i64,
    history:bool,
    tables:BTreeMap<String,Vec<Vec<Value>>>,
    settings:BTreeMap<String,String>,
}

fn directory()->Result<PathBuf,&'static str> {
    let path=crate::data_dir().map_err(|_|"backup_folder_unavailable")?.join("backups");
    fs::create_dir_all(&path).map_err(|_|"backup_folder_unavailable")?;
    Ok(path)
}
fn key(passphrase:&str,salt:&[u8])->Result<aead::LessSafeKey,&'static str> {
    let mut bytes=Zeroizing::new([0u8;32]);
    pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA256,NonZeroU32::new(ITERATIONS).unwrap(),salt,
        passphrase.as_bytes(),bytes.as_mut());
    Ok(aead::LessSafeKey::new(aead::UnboundKey::new(&aead::AES_256_GCM,bytes.as_ref())
        .map_err(|_|"backup_encryption_failed")?))
}
fn validate_passphrase(value:&str)->Result<(),&'static str> {
    if value.chars().count()<12 || value.len()>1024 {Err("backup_passphrase_needs_12_characters")}
    else {Ok(())}
}
fn seal(snapshot:&Snapshot,passphrase:&str)->Result<Vec<u8>,&'static str> {
    seal_payload(MAGIC,serde_json::to_vec(snapshot).map_err(|_|"backup_encoding_failed")?,passphrase)
}
pub(crate) fn seal_payload(magic:&[u8],bytes:Vec<u8>,passphrase:&str)->Result<Vec<u8>,&'static str> {
    validate_passphrase(passphrase)?;
    let mut payload=Zeroizing::new(bytes);
    if payload.len()>MAX_BYTES-256 {return Err("backup_too_large_choose_settings_only");}
    let mut salt=[0;SALT_BYTES];
    let mut nonce=[0;NONCE_BYTES];
    let rng=SystemRandom::new();
    rng.fill(&mut salt).map_err(|_|"backup_encryption_failed")?;
    rng.fill(&mut nonce).map_err(|_|"backup_encryption_failed")?;
    let mut wire=magic.to_vec();
    wire.extend_from_slice(&salt);
    wire.extend_from_slice(&nonce);
    key(passphrase,&salt)?.seal_in_place_append_tag(aead::Nonce::assume_unique_for_key(nonce),
        aead::Aad::from(wire.as_slice()),&mut *payload).map_err(|_|"backup_encryption_failed")?;
    wire.extend_from_slice(&payload);
    Ok(wire)
}
fn open(wire:&[u8],passphrase:&str)->Result<Snapshot,&'static str> {
    serde_json::from_slice(&open_payload(MAGIC,wire,passphrase)?).map_err(|_|"invalid_backup_data")
}
pub(crate) fn open_payload(magic:&[u8],wire:&[u8],passphrase:&str)->Result<Zeroizing<Vec<u8>>,&'static str> {
    validate_passphrase(passphrase)?;
    let n=magic.len();
    let header=n+SALT_BYTES+NONCE_BYTES;
    if wire.len()>MAX_BYTES || wire.len()<header+16 || !wire.starts_with(magic) {
        return Err("invalid_backup_format");
    }
    let salt=&wire[n..n+SALT_BYTES];
    let nonce:[u8;NONCE_BYTES]=wire[n+SALT_BYTES..header].try_into().unwrap();
    let mut payload=Zeroizing::new(wire[header..].to_vec());
    let length=key(passphrase,salt)?.open_in_place(aead::Nonce::assume_unique_for_key(nonce),
        aead::Aad::from(&wire[..header]),&mut payload).map_err(|_|"wrong_passphrase_or_damaged_backup")?.len();
    payload.truncate(length);
    Ok(payload)
}
fn tables(history:bool)->impl Iterator<Item=&'static str> {
    CONFIG.iter().copied().chain(HISTORY.iter().copied().filter(move |_|history))
}
fn capture(db:&Connection,history:bool,now:i64)->Result<Snapshot,&'static str> {
    let tx=db.unchecked_transaction().map_err(|_|"backup_snapshot_unavailable")?;
    let mut snapshot=Snapshot{storage_version:crate::schema::STORAGE_VERSION,created_ms:now,
        history,tables:BTreeMap::new(),settings:BTreeMap::new()};
    let mut length=1024;
    for table in tables(history) {
        let mut q=tx.prepare(&format!("SELECT * FROM {table}")).map_err(|_|"backup_snapshot_unavailable")?;
        let columns=q.column_count();
        let mut cursor=q.query([]).map_err(|_|"backup_snapshot_unavailable")?;
        let mut rows=Vec::new();
        while let Some(row)=cursor.next().map_err(|_|"backup_snapshot_unavailable")? {
            let mut fields=Vec::with_capacity(columns);
            for column in 0..columns {
                fields.push(match row.get_ref(column).map_err(|_|"backup_snapshot_unavailable")? {
                    ValueRef::Null=>Value::Null,
                    ValueRef::Integer(n)=>json!(n),
                    ValueRef::Real(n)=>json!(n),
                    ValueRef::Text(text)=>Value::String(std::str::from_utf8(text).map_err(|_|"invalid_local_data")?.to_owned()),
                    ValueRef::Blob(_)=>return Err("unsupported_backup_field"),
                });
            }
            length+=serde_json::to_vec(&fields).map_err(|_|"backup_encoding_failed")?.len()+1;
            if length>MAX_BYTES-65536 {return Err("backup_too_large_choose_settings_only");}
            rows.push(fields);
        }
        snapshot.tables.insert(table.to_owned(),rows);
    }
    for name in SETTINGS {
        let value=features::read_setting(&tx,name);
        if !value.is_empty() {snapshot.settings.insert((*name).into(),value);}
    }
    tx.commit().map_err(|_|"backup_snapshot_unavailable")?;
    Ok(snapshot)
}

fn read_database(path:&str)->Result<Connection,&'static str> {
    let db=Connection::open_with_flags(path,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|_|"backup_snapshot_unavailable")?;
    db.busy_timeout(std::time::Duration::from_secs(3)).map_err(|_|"backup_snapshot_unavailable")?;
    Ok(db)
}
fn file_name_valid(name:&str)->bool {
    name.len()<120 && name.ends_with(".ddbackup") && name.as_bytes().iter().all(|byte|
        byte.is_ascii_alphanumeric() || b"-_.".contains(byte)) && !name.contains("..")
}
fn read_backup(dir:&Path,name:&str)->Result<Vec<u8>,&'static str> {
    if !file_name_valid(name) {return Err("invalid_backup_name");}
    let path=dir.join(name);
    let meta=fs::symlink_metadata(&path).map_err(|_|"backup_file_unavailable")?;
    if !meta.file_type().is_file() || meta.len()>MAX_BYTES as u64 {return Err("invalid_backup_file");}
    let mut wire=Vec::new();
    File::open(path).map_err(|_|"backup_file_unavailable")?.take((MAX_BYTES+1) as u64)
        .read_to_end(&mut wire).map_err(|_|"backup_read_failed")?;
    if wire.len()>MAX_BYTES {return Err("backup_file_too_large");}
    Ok(wire)
}
fn save_file(dir:&Path,prefix:&str,wire:&[u8])->Result<String,&'static str> {
    let name=format!("{prefix}-{}-{}.ddbackup",now_ms(),Uuid::new_v4());
    let partial=dir.join(format!("{name}.partial"));
    let result=(|| -> std::io::Result<()> {
        let mut file=OpenOptions::new().write(true).create_new(true).open(&partial)?;
        file.write_all(wire)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&partial,dir.join(&name))
    })();
    if result.is_err() {let _=fs::remove_file(partial);return Err("backup_write_failed");}
    Ok(name)
}

fn sql_value(value:&Value)->Result<SqlValue,&'static str> {
    match value {
        Value::Null=>Ok(SqlValue::Null),
        Value::Number(n)=>n.as_i64().map(SqlValue::Integer).or_else(||n.as_f64().map(SqlValue::Real))
            .ok_or("invalid_backup_number"),
        Value::String(s) if s.len()<=262_144=>Ok(SqlValue::Text(s.clone())),
        _=>Err("invalid_backup_field"),
    }
}

fn apply(db:&Connection,snapshot:&Snapshot,now:i64)->Result<(),&'static str> {
    if snapshot.storage_version!=crate::schema::STORAGE_VERSION || snapshot.created_ms<=0
        || snapshot.created_ms>now+600_000
        || snapshot.tables.keys().map(String::as_str).collect::<BTreeSet<_>>()!=tables(snapshot.history).collect()
        || snapshot.settings.keys().any(|key|!SETTINGS.contains(&key.as_str()))
        || snapshot.settings.values().any(|value|value.len()>65536)
    {return Err("incompatible_backup_schema");}
    if snapshot.history {
        let cutoff=features::read_setting(db,"activity_deleted_before_ms").parse::<i64>().unwrap_or(0);
        let scoped=db.query_row("SELECT COALESCE(MAX(before_ms),0) FROM purged_subjects",[],|r|r.get::<_,i64>(0))
            .map_err(|_|"backup_privacy_check_failed")?;
        if snapshot.created_ms<cutoff.max(scoped) {return Err("backup_predates_activity_deletion");}
    }
    let tx=db.unchecked_transaction().map_err(|_|"restore_storage_failed")?;
    let selected=tables(snapshot.history).collect::<Vec<_>>();
    // Exclude companion identity, credentials, pairing, and device enrollment.
    for table in selected.iter().rev() {
        tx.execute(&format!("DELETE FROM {table}"),[]).map_err(|_|"restore_storage_failed")?;
    }
    for table in selected {
        let columns=tx.prepare(&format!("SELECT * FROM {table} LIMIT 0")).map_err(|_|"restore_storage_failed")?.column_count();
        let placeholders=vec!["?";columns].join(",");
        let mut insert=tx.prepare(&format!("INSERT INTO {table} VALUES({placeholders})"))
            .map_err(|_|"restore_storage_failed")?;
        for row in &snapshot.tables[table] {
            if row.len()!=columns {return Err("invalid_backup_columns");}
            let values=row.iter().map(sql_value).collect::<Result<Vec<_>,_>>()?;
            insert.execute(params_from_iter(values)).map_err(|_|"invalid_backup_rows")?;
        }
    }
    {
        let mut query=tx.prepare("SELECT domain,path_prefix,config_json,budget_minutes FROM rules")
            .map_err(|_|"restore_storage_failed")?;
        let mut rows=query.query([]).map_err(|_|"restore_storage_failed")?;
        while let Some(row)=rows.next().map_err(|_|"restore_storage_failed")? {
            let domain:String=row.get(0).map_err(|_|"invalid_backup_rule")?;
            let path:Option<String>=row.get(1).map_err(|_|"invalid_backup_rule")?;
            let config:String=row.get(2).map_err(|_|"invalid_backup_rule")?;
            let budget:Option<u32>=row.get(3).map_err(|_|"invalid_backup_rule")?;
            if !crate::domain::valid_domain(&domain)
                || path.as_deref().is_some_and(|path|!crate::domain::safe_path(path))
                || budget.is_some_and(|value|!(1..=1440).contains(&value))
                || !serde_json::from_str::<crate::policy::RuleConfig>(&config).is_ok_and(|rule|rule.validate())
            {return Err("invalid_backup_rule");}
        }
    }
    for (key,value) in &snapshot.settings {
        tx.execute("INSERT OR REPLACE INTO settings(key,value) VALUES(?1,?2)",params![key,value])
            .map_err(|_|"restore_storage_failed")?;
    }
    if snapshot.history {
        tx.execute("UPDATE focus_sessions SET ended_ms=MIN(end_ms,?1) WHERE ended_ms IS NULL",[now])
            .map_err(|_|"restore_storage_failed")?;
        tx.execute("UPDATE focus_cycles SET state='finished' WHERE state!='finished'",[])
            .map_err(|_|"restore_storage_failed")?;
        tx.execute("UPDATE reading_windows SET stopped=1",[]).map_err(|_|"restore_storage_failed")?;
        tx.execute("UPDATE reward_days SET cursor_ms=MAX(cursor_ms,?1),pending_ms=0",[now])
            .map_err(|_|"restore_storage_failed")?;
        tx.execute("INSERT OR REPLACE INTO settings VALUES('reward_enabled_at_ms',?1)",[now.to_string()])
            .map_err(|_|"restore_storage_failed")?;
        tx.execute("UPDATE sessions SET profile_id=NULL",[]).map_err(|_|"restore_storage_failed")?;
    }
    tx.execute_batch("DELETE FROM rule_states; DELETE FROM temporary_exceptions; DELETE FROM learning_sessions;")
        .map_err(|_|"restore_storage_failed")?;
    crate::data_lifecycle::advance_epoch(&tx).map_err(|_|"restore_storage_failed")?;
    tx.commit().map_err(|_|"restore_storage_failed")
}
fn validate(snapshot:&Snapshot,now:i64)->Result<(),&'static str> {
    let db=Connection::open_in_memory().map_err(|_|"restore_validation_unavailable")?;
    crate::initialize_schema(&db).map_err(|_|"restore_validation_unavailable")?;
    apply(&db,snapshot,now)
}
fn failure(error:&str)->(StatusCode,String) {
    err(StatusCode::BAD_REQUEST,error)
}
async fn work(action:impl FnOnce()->Result<Value,&'static str>+Send+'static)->ApiResult {
    let result=tokio::task::spawn_blocking(action).await
        .map_err(|_|err(StatusCode::INTERNAL_SERVER_ERROR,"backup_worker_failed"))?;
    result.map(Json).map_err(failure)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Create {passphrase:String,history:bool}
pub async fn create(State(state):State<Arc<AppState>>,Json(input):Json<Create>)->ApiResult {
    work(move||{
        let _lease=Lease::take()?;
        let path=state.db.lock().unwrap().path().ok_or("backup_snapshot_unavailable")?.to_owned();
        let passphrase=Zeroizing::new(input.passphrase);
        validate_passphrase(&passphrase)?;
        let snapshot=capture(&read_database(&path)?,input.history,now_ms())?;
        let wire=seal(&snapshot,&passphrase)?;
        let filename=save_file(&directory()?,"backup",&wire)?;
        remember_success(&state,now_ms());
        Ok(json!({"ok":true,"filename":filename,"bytes":wire.len()}))
    }).await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Import {filename:String,passphrase:String,digest:Option<String>}
pub async fn preview(Json(input):Json<Import>)->ApiResult {
    work(move||{
        let _lease=Lease::take()?;
        let secret=Zeroizing::new(input.passphrase);
        let wire=read_backup(&directory()?,&input.filename)?;
        let snapshot=open(&wire,&secret)?;
        validate(&snapshot,now_ms())?;
        let rows=snapshot.tables.iter().map(|(key,rows)|(key,rows.len())).collect::<BTreeMap<_,_>>();
        Ok(json!({"ok":true,"digest":hex::encode(Sha256::digest(&wire)),
            "created_ms":snapshot.created_ms,"history":snapshot.history,"rows":rows}))
    }).await
}

struct PauseRestore<'a>{state:&'a AppState,previous:bool}
impl Drop for PauseRestore<'_> {
    fn drop(&mut self){self.state.paused.store(self.previous,Ordering::SeqCst);}
}
pub async fn restore(State(state):State<Arc<AppState>>,Json(input):Json<Import>)->ApiResult {
    work(move||{
        let _lease=Lease::take()?;
        let secret=Zeroizing::new(input.passphrase);
        let dir=directory()?;
        let wire=read_backup(&dir,&input.filename)?;
        if input.digest.as_deref()!=Some(hex::encode(Sha256::digest(&wire)).as_str()){
            return Err("preview_the_current_backup_before_restoring");
        }
        let snapshot=open(&wire,&secret)?;
        validate(&snapshot,now_ms())?;
        let previous=state.paused.swap(true,Ordering::SeqCst);
        let _pause=PauseRestore{state:&state,previous};
        crate::stop_observing(&state);
        let path=state.db.lock().unwrap().path().ok_or("backup_snapshot_unavailable")?.to_owned();
        let recovery=capture(&read_database(&path)?,true,now_ms())?;
        let recovery=save_file(&dir,"before-restore",&seal(&recovery,&secret)?)?;
        apply(&state.db.lock().unwrap(),&snapshot,now_ms())?;
        Ok(json!({"ok":true,"recovery":recovery}))
    }).await
}

fn remember_success(state:&AppState,at:i64) {
    let _=state.db.lock().unwrap().execute("INSERT OR REPLACE INTO settings VALUES('backup_last_ok_ms',?1)",[at.to_string()]);
}
pub async fn status(State(state):State<Arc<AppState>>)->ApiResult {
    work(move||{
        let dir=directory()?;
        let mut files=Vec::new();
        for item in fs::read_dir(&dir).map_err(|_|"backup_folder_unavailable")?.flatten() {
            let name=item.file_name().to_string_lossy().into_owned();
            if !file_name_valid(&name) || !item.file_type().is_ok_and(|kind|kind.is_file()) {continue;}
            let Ok(metadata)=item.metadata() else {continue;};
            files.push((name,metadata.len()));
        }
        files.sort_by(|a,b|b.0.cmp(&a.0));
        files.truncate(150);
        let db=state.db.lock().unwrap();
        Ok(json!({"ok":true,"folder":dir,"files":files,
            "last_backup_ms":features::read_setting(&db,"backup_last_ok_ms").parse::<i64>().ok(),
            "scheduled":features::read_setting(&db,"backup_daily")=="true"}))
    }).await
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Schedule{enabled:bool,passphrase:Option<String>,history:bool}
pub async fn schedule(State(state):State<Arc<AppState>>,Json(input):Json<Schedule>)->ApiResult {
    work(move||{
        let _lease=Lease::take()?;
        let key_path=crate::data_dir().map_err(|_|"backup_folder_unavailable")?.join("backup-schedule.key");
        if input.enabled {
            let secret=Zeroizing::new(input.passphrase.ok_or("backup_passphrase_required")?);
            validate_passphrase(&secret)?;
            let protected=crate::credentials::protect_secret(secret.as_bytes()).map_err(|_|"backup_schedule_requires_windows")?;
            fs::write(&key_path,protected).map_err(|_|"backup_schedule_storage_failed")?;
        }
        let db=state.db.lock().unwrap();
        let tx=db.unchecked_transaction().map_err(|_|"backup_schedule_storage_failed")?;
        for (key,value) in [("backup_daily",input.enabled),("backup_daily_history",input.history)] {
            tx.execute("INSERT OR REPLACE INTO settings VALUES(?1,?2)",params![key,value.to_string()])
                .map_err(|_|"backup_schedule_storage_failed")?;
        }
        tx.commit().map_err(|_|"backup_schedule_storage_failed")?;
        if !input.enabled {let _=fs::remove_file(key_path);}
        Ok(json!({"ok":true}))
    }).await
}

pub async fn scheduler(state:Arc<AppState>) {
    let mut interval=tokio::time::interval(std::time::Duration::from_secs(300));
    loop {
        interval.tick().await;
        if state.stopping.load(Ordering::SeqCst) {return;}
        let state=state.clone();
        let _=tokio::task::spawn_blocking(move|| {
            let now=now_ms();
            let job={
                let db=state.db.lock().unwrap();
                let last=features::read_setting(&db,"backup_last_scheduled_ms").parse::<i64>().unwrap_or(0);
                if features::read_setting(&db,"backup_daily")!="true" || now-last<86_400_000 {return;}
                (db.path().unwrap_or_default().to_owned(),features::read_setting(&db,"backup_daily_history")=="true")
            };
            let Ok(_lease)=Lease::take() else{return;};
            let result=(|| -> Result<(),&'static str> {
                let key_path=crate::data_dir().map_err(|_|"backup_folder_unavailable")?.join("backup-schedule.key");
                let sealed=fs::read(key_path).map_err(|_|"backup_schedule_key_missing")?;
                let secret=Zeroizing::new(crate::credentials::unprotect_secret(&sealed)
                    .map_err(|_|"backup_schedule_key_unavailable")?);
                let passphrase=std::str::from_utf8(&secret).map_err(|_|"backup_schedule_key_invalid")?;
                let dir=directory()?;
                let snapshot=capture(&read_database(&job.0)?,job.1,now)?;
                save_file(&dir,"scheduled",&seal(&snapshot,passphrase)?)?;
                let mut old=fs::read_dir(&dir).into_iter().flatten().flatten().filter_map(|file|{
                    let name=file.file_name().to_string_lossy().into_owned();
                    (name.starts_with("scheduled-") && file_name_valid(&name)
                        && file.file_type().is_ok_and(|kind|kind.is_file())).then_some(name)
                }).collect::<Vec<_>>();
                old.sort();
                for name in old.iter().take(old.len().saturating_sub(7)) {let _=fs::remove_file(dir.join(name));}
                Ok(())
            })();
            if result.is_ok() {remember_success(&state,now);}
            let db=state.db.lock().unwrap();
            let _=db.execute("INSERT OR REPLACE INTO settings VALUES('backup_last_scheduled_ms',?1)",[now.to_string()]);
            if let Err(code)=result {
                let _=crate::inbox::record(&db,"backup:schedule","health","Scheduled backup needs attention",
                    json!({"code":code}),"Settings & Privacy",None,now);
            }
        }).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encrypted_backup_round_trips_without_secrets_and_rejects_modified_bytes() {
        let db=Connection::open_in_memory().unwrap();
        crate::initialize_schema(&db).unwrap();
        db.execute("INSERT INTO settings VALUES('provider_secret','private-test-value')",[]).unwrap();
        db.execute("INSERT INTO category_overrides VALUES('app:editor.exe','Design',1)",[]).unwrap();
        let snapshot=capture(&db,true,now_ms()).unwrap();
        assert!(!serde_json::to_string(&snapshot).unwrap().contains("private-test-value"));
        let mut wire=seal(&snapshot,"twelve-plus-characters").unwrap();
        let restored=open(&wire,"twelve-plus-characters").unwrap();
        let target=Connection::open_in_memory().unwrap();
        crate::initialize_schema(&target).unwrap();
        apply(&target,&restored,now_ms()).unwrap();
        assert_eq!(features::category_for(&target,"editor.exe",None),"Design");
        assert!(open(&wire,"incorrect-passphrase").is_err());
        let last=wire.len()-1;
        wire[last]^=1;
        assert!(open(&wire,"twelve-plus-characters").is_err());
        target.execute("INSERT OR REPLACE INTO settings VALUES('activity_deleted_before_ms',?1)",
            [(restored.created_ms+1000).to_string()]).unwrap();
        assert_eq!(apply(&target,&restored,now_ms()),Err("backup_predates_activity_deletion"));
    }
}

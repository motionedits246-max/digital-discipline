//! Optional earned browser time, accounted once from local foreground observations.
use crate::{domain, err, features, midnight_local_ms, now_ms, ApiResult, AppState};
use axum::{extract::State, http::StatusCode, Json};
use chrono::{Duration, Local, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

const MINUTE: i64 = 60_000;
const RECORDING_SETTLE_MS: i64 = 4_000;

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Terms {
    enabled: bool,
    work_apps: Vec<String>,
    work_domains: Vec<String>,
    spend_domains: Vec<String>,
    work_minutes: u32,
    reward_minutes: u32,
    daily_cap_minutes: u32,
    rollover: bool,
}

impl Default for Terms {
    fn default() -> Self {
        Self { enabled:false, work_apps:vec![], work_domains:vec![], spend_domains:vec![],
            work_minutes:45, reward_minutes:10, daily_cap_minutes:30, rollover:false }
    }
}

pub fn migrate(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS reward_days(
            day TEXT PRIMARY KEY, cursor_ms INTEGER NOT NULL, pending_ms INTEGER NOT NULL DEFAULT 0,
            earned_ms INTEGER NOT NULL DEFAULT 0, spent_ms INTEGER NOT NULL DEFAULT 0,
            carried_ms INTEGER NOT NULL DEFAULT 0, adjusted_ms INTEGER NOT NULL DEFAULT 0,
            closed INTEGER NOT NULL DEFAULT 0);
         CREATE TABLE IF NOT EXISTS reward_ledger(
            id INTEGER PRIMARY KEY, day TEXT NOT NULL, at_ms INTEGER NOT NULL,
            kind TEXT NOT NULL, amount_ms INTEGER NOT NULL);",
    )
}

fn terms(db: &Connection) -> Terms {
    serde_json::from_str(&features::read_setting(db,"reward_terms")).unwrap_or_default()
}
fn balance(earned: i64, spent: i64, carried: i64, adjusted: i64) -> i64 {
    (earned+carried+adjusted-spent).max(0)
}
fn storage(_: rusqlite::Error) -> (StatusCode,String) {
    err(StatusCode::INTERNAL_SERVER_ERROR,"reward_storage_failed")
}
fn local_day(now:i64)->rusqlite::Result<(String,i64,i64)> {
    let day=chrono::DateTime::<Utc>::from_timestamp_millis(now).ok_or(rusqlite::Error::InvalidQuery)?
        .with_timezone(&Local).date_naive();
    Ok((day.to_string(),midnight_local_ms(day),midnight_local_ms(day+Duration::days(1))))
}
fn entry(db:&Connection,day:&str,kind:&str,amount:i64,now:i64)->rusqlite::Result<()> {
    if amount!=0 {
        db.execute("INSERT INTO reward_ledger(day,at_ms,kind,amount_ms) VALUES(?1,?2,?3,?4)",
            params![day,now,kind,amount])?;
    }
    Ok(())
}

#[derive(Default)]
struct DayBalance {
    cursor: i64, pending: i64, earned: i64, spent: i64, carried: i64, adjusted: i64, closed: bool,
}
fn read_day(db:&Connection,day:&str)->rusqlite::Result<Option<DayBalance>> {
    db.query_row("SELECT cursor_ms,pending_ms,earned_ms,spent_ms,carried_ms,adjusted_ms,closed FROM reward_days WHERE day=?1",
        [day], |r|Ok(DayBalance {cursor:r.get(0)?,pending:r.get(1)?,earned:r.get(2)?,spent:r.get(3)?,
            carried:r.get(4)?,adjusted:r.get(5)?,closed:r.get(6)?})).optional()
}
fn host_matches(targets:&[String],host:Option<&str>)->bool {
    host.is_some_and(|name| targets.iter().any(|item|name == item || name.strip_suffix(item).is_some_and(|prefix|prefix.ends_with('.'))))
}

fn account(db:&Connection,terms:&Terms,now:i64)->rusqlite::Result<(String,DayBalance)> {
    let (day,start,end)=local_day(now)?;
    let tx=db.unchecked_transaction()?;
    if read_day(&tx,&day)?.is_none() {
        let previous={
            let mut q=tx.prepare("SELECT day,earned_ms,spent_ms,carried_ms,adjusted_ms FROM reward_days
                WHERE closed=0 AND day<?1 ORDER BY day DESC")?;
            let rows=q.query_map([&day],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?,
                r.get::<_,i64>(2)?,r.get::<_,i64>(3)?,r.get::<_,i64>(4)?)))?;
            rows.collect::<rusqlite::Result<Vec<_>>>()?
        };
        let mut carry=0;
        for (index,(old,earned,spent,carried,adjusted)) in previous.into_iter().enumerate() {
            let available=balance(earned,spent,carried,adjusted);
            let accepted=if index==0 && terms.rollover { available.min(terms.daily_cap_minutes as i64*MINUTE) } else {0};
            carry+=accepted;
            entry(&tx,&old,"expired",available-accepted,now)?;
            entry(&tx,&old,"rolled_out",accepted,now)?;
            tx.execute("UPDATE reward_days SET closed=1 WHERE day=?1",[old])?;
        }
        tx.execute("INSERT INTO reward_days(day,cursor_ms,carried_ms) VALUES(?1,?2,?3)",
            params![day,start,carry])?;
        entry(&tx,&day,"rolled_in",carry,now)?;
    }
    let mut value=read_day(&tx,&day)?.ok_or(rusqlite::Error::InvalidQuery)?;
    let settled=(now-RECORDING_SETTLE_MS).min(end);
    if value.closed || !terms.enabled || settled<=value.cursor {
        tx.commit()?;
        return Ok((day,value));
    }
    let enabled_at=features::read_setting(&tx,"reward_enabled_at_ms").parse::<i64>().unwrap_or(now);
    let from=value.cursor.max(start).max(enabled_at);
    let work=terms.work_minutes as i64*MINUTE;
    let payout=terms.reward_minutes as i64*MINUTE;
    let cap=terms.daily_cap_minutes as i64*MINUTE;
    let mut earned=0;
    let mut spent=0;
    if settled>from {
        let mut rows=tx.prepare("SELECT start_ms,end_ms,app,domain FROM sessions
            WHERE idle=0 AND end_ms>?1 AND start_ms<?2 ORDER BY start_ms,id")?;
        let mut matches=rows.query(params![from,settled])?;
        let mut covered_until=from;
        while let Some(row)=matches.next()? {
            let lo=row.get::<_,i64>(0)?.max(covered_until);
            let hi=row.get::<_,i64>(1)?.min(settled);
            if hi<=lo {continue;}
            let app:String=row.get(2)?;
            let host:Option<String>=row.get(3)?;
            covered_until=hi;
            if features::excluded(&tx,&app,host.as_deref()) {continue;}
            let elapsed=hi-lo;
            if host_matches(&terms.spend_domains,host.as_deref()) {
                let debit=elapsed.min(balance(value.earned,value.spent,value.carried,value.adjusted));
                value.spent+=debit;
                spent+=debit;
            } else if terms.work_apps.iter().any(|item|item.eq_ignore_ascii_case(&app))
                || host_matches(&terms.work_domains,host.as_deref()) {
                value.pending+=elapsed;
                let batches=value.pending/work;
                value.pending%=work;
                let credit=(batches*payout).min((cap-value.earned).max(0));
                value.earned+=credit;
                earned+=credit;
            }
        }
    }
    value.cursor=settled;
    tx.execute("UPDATE reward_days SET cursor_ms=?2,pending_ms=?3,earned_ms=?4,spent_ms=?5 WHERE day=?1",
        params![day,value.cursor,value.pending,value.earned,value.spent])?;
    entry(&tx,&day,"earned",earned,now)?;
    entry(&tx,&day,"spent",spent,now)?;
    tx.commit()?;
    Ok((day,value))
}

pub fn reset(db:&Connection,now:i64)->rusqlite::Result<()> {
    db.execute_batch("DELETE FROM reward_days; DELETE FROM reward_ledger;")?;
    db.execute("INSERT OR REPLACE INTO settings VALUES('reward_enabled_at_ms',?1)",[now.to_string()])?;
    Ok(())
}

pub async fn save(State(state):State<Arc<AppState>>,Json(input):Json<Terms>)->ApiResult {
    if !(1..=240).contains(&input.work_minutes) || !(1..=120).contains(&input.reward_minutes)
        || !(1..=240).contains(&input.daily_cap_minutes) || input.reward_minutes>input.daily_cap_minutes
        || input.work_apps.len()>20 || input.work_domains.len()>20 || input.spend_domains.len()>20
        || input.work_apps.iter().any(|name|name.is_empty()||name.len()>100||!name.bytes().all(|byte|
            byte.is_ascii_alphanumeric() || b"_.-".contains(&byte)))
        || input.work_domains.iter().chain(&input.spend_domains).any(|name|!domain::valid_domain(name))
        || input.enabled && (input.spend_domains.is_empty() || input.work_apps.is_empty() && input.work_domains.is_empty())
    {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_reward_terms"));
    }
    let db=state.db.lock().unwrap();
    let now=now_ms();
    account(&db,&terms(&db),now).map_err(storage)?;
    let tx=db.unchecked_transaction().map_err(storage)?;
    tx.execute("INSERT OR REPLACE INTO settings VALUES('reward_terms',?1)",
        [serde_json::to_string(&input).unwrap()]).map_err(storage)?;
    tx.execute("INSERT OR REPLACE INTO settings VALUES('reward_enabled_at_ms',?1)",[now.to_string()]).map_err(storage)?;
    tx.execute("UPDATE reward_days SET pending_ms=0 WHERE closed=0",[]).map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(Json(json!({"ok":true})))
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Adjustment { minutes: i32 }
pub async fn adjust(State(state):State<Arc<AppState>>,Json(input):Json<Adjustment>)->ApiResult {
    if input.minutes==0 || input.minutes.unsigned_abs()>240 {
        return Err(err(StatusCode::BAD_REQUEST,"invalid_reward_adjustment"));
    }
    let db=state.db.lock().unwrap();
    let now=now_ms();
    let terms=terms(&db);
    let (day,amounts)=account(&db,&terms,now).map_err(storage)?;
    let delta=input.minutes as i64*MINUTE;
    let next=balance(amounts.earned,amounts.spent,amounts.carried,amounts.adjusted)+delta;
    if amounts.closed || !terms.enabled || !(0..=terms.daily_cap_minutes as i64*MINUTE).contains(&next) {
        return Err(err(StatusCode::BAD_REQUEST,"reward_adjustment_out_of_range"));
    }
    let tx=db.unchecked_transaction().map_err(storage)?;
    tx.execute("UPDATE reward_days SET adjusted_ms=adjusted_ms+?2 WHERE day=?1",params![day,delta]).map_err(storage)?;
    entry(&tx,&day,"adjusted",delta,now).map_err(storage)?;
    tx.commit().map_err(storage)?;
    Ok(Json(json!({"ok":true})))
}

pub async fn status(State(state):State<Arc<AppState>>)->ApiResult {
    let db=state.db.lock().unwrap();
    let terms=terms(&db);
    let (day,amounts)=account(&db,&terms,now_ms()).map_err(storage)?;
    let mut q=db.prepare("SELECT id,day,at_ms,kind,amount_ms FROM reward_ledger ORDER BY id DESC LIMIT 80").map_err(storage)?;
    let ledger=q.query_map([],|r|Ok(json!({"id":r.get::<_,i64>(0)?,"day":r.get::<_,String>(1)?,
        "at_ms":r.get::<_,i64>(2)?,"kind":r.get::<_,String>(3)?,"amount_ms":r.get::<_,i64>(4)?})))
        .map_err(storage)?.collect::<rusqlite::Result<Vec<_>>>().map_err(storage)?;
    Ok(Json(json!({"ok":true,"terms":terms,"day":day,"pending_ms":amounts.pending,
        "balance_ms":if amounts.closed {0}else{balance(amounts.earned,amounts.spent,amounts.carried,amounts.adjusted)},
        "earned_ms":amounts.earned,"spent_ms":amounts.spent,"entries":ledger})))
}

pub fn browser_policy(db:&Connection,now:i64)->rusqlite::Result<(Vec<Value>,Option<i64>)> {
    let terms=terms(db);
    if !terms.enabled {return Ok((Vec::new(),None));}
    let (_,amounts)=account(db,&terms,now)?;
    let (_,_,end)=local_day(now)?;
    let balance=if amounts.closed {0}else{balance(amounts.earned,amounts.spent,amounts.carried,amounts.adjusted)};
    let check=(now+balance.clamp(5_000,60_000)).min(end);
    let blocks=if balance==0 { terms.spend_domains.iter().map(|host|json!({
        "id":format!("earned:{host}"),"domain":host,"path_prefix":null,"action":"block",
        "reason":"Earned entertainment balance is empty","expires_at_ms":(now+65_000).min(end)
    })).collect() }else{Vec::new()};
    Ok((blocks,Some(check)))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn observed_intervals_earn_once_and_spending_cannot_overdraw() {
        let db=Connection::open_in_memory().unwrap();
        crate::initialize_schema(&db).unwrap();
        let (_,start,_)=local_day(now_ms()).unwrap();
        let now=start+300_000;
        db.execute("INSERT OR REPLACE INTO settings VALUES('reward_enabled_at_ms',?1)",[start.to_string()]).unwrap();
        for (id,a,b,app,idle,host) in [
            ("work",1,121_000,"editor.exe",0,None),
            ("overlap",30_000,80_000,"editor.exe",0,None),
            ("idle",125_000,160_000,"editor.exe",1,None),
            ("break",170_000,280_000,"brave.exe",0,Some("youtube.com")),
        ] {
            db.execute("INSERT INTO sessions(id,start_ms,end_ms,app,idle,domain) VALUES(?1,?2,?3,?4,?5,?6)",
                params![id,start+a,start+b,app,idle,host]).unwrap();
        }
        let terms=Terms{enabled:true,work_apps:vec!["editor.exe".into()],spend_domains:vec!["youtube.com".into()],
            work_minutes:2,reward_minutes:1,daily_cap_minutes:2,..Terms::default()};
        for _ in 0..2 {
            let (_,v)=account(&db,&terms,now).unwrap();
            assert_eq!((v.earned,v.spent),(60_000,60_000));
        }
        assert_eq!(db.query_row("SELECT count(*) FROM reward_ledger",[],|r|r.get::<_,i64>(0)).unwrap(),2);
    }
}

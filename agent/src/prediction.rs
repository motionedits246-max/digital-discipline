//! Conservative, optional, purely local empirical habit forecasts.
//! This is not a clinical measure, nor a prediction of any individual's behavior.
//! A forecast appears only when a minimum sample size AND out-of-sample calibration
//! outperform an hour-of-day independent baseline. No remote calls are made.
use chrono::{DateTime, Local, TimeZone, Timelike, NaiveDate};
use rusqlite::{params, Connection};
use serde_json::{json,Value};
use std::collections::{BTreeMap, BTreeSet};

const MIN_DAYS: usize = 21;
const MIN_OBS_PER_HOUR: usize = 12;
const MIN_HOUR_COVERAGE_MS: i64 = 5*60_000;

#[derive(Clone,Copy,Debug)]
struct Sample { day:NaiveDate, hour:u32, distraction:bool }

/// Uses completed, observed hourly buckets. Session coverage is deliberately capped
/// and merged; an hour with under 5 minutes of known foreground activity is omitted.
/// The signal is whether the user-selected distraction category appeared at all,
/// NOT a statement that any particular website or activity is objectively harmful.
pub fn forecast(db:&Connection, now:DateTime<Local>, enabled:bool, categories:&[String],
    resolve:impl Fn(&str,Option<&str>)->String)->rusqlite::Result<Value> {
    if !enabled {return Ok(json!({"available":false,"reason":"Enable local prediction in Settings & Privacy to evaluate your own activity patterns."}));}
    if categories.is_empty() {return Ok(json!({"available":false,"reason":"Choose your own distraction categories first; nothing is labeled automatically."}));}
    let start=(now-chrono::Duration::days(62)).timestamp_millis();
    let mut q=db.prepare("SELECT start_ms,end_ms,app,domain FROM sessions WHERE idle=0 AND end_ms>?1 AND start_ms<?2 ORDER BY start_ms LIMIT 350000")?;
    let rows=q.query_map(params![start,now.timestamp_millis()],|r|Ok((r.get::<_,i64>(0)?,r.get::<_,i64>(1)?,r.get::<_,String>(2)?,r.get::<_,Option<String>>(3)?)))?;
    let mut buckets:BTreeMap<(NaiveDate,u32),Vec<(i64,i64,bool)>>=BTreeMap::new();
    let current_day=now.date_naive();
    for row in rows {
        let (from,to,app,domain)=row?;
        if to<=from {continue;}
        let flagged=categories.contains(&resolve(&app,domain.as_deref()));
        let mut cursor=from.max(start);
        let limit=to.min(now.timestamp_millis());
        let mut split_count=0;
        while cursor<limit && split_count<1600 {
            let Some(t)=DateTime::<chrono::Utc>::from_timestamp_millis(cursor).map(|x|x.with_timezone(&Local)) else {break;};
            let day=t.date_naive();let hour=t.hour();
            if day==current_day {break;} // never use partly observed current day to fit
            let next_naive=(day.and_hms_opt(hour,0,0).unwrap()+chrono::Duration::hours(1));
            let boundary=Local.from_local_datetime(&next_naive).earliest()
                .or_else(||Local.from_local_datetime(&next_naive).latest())
                .map(|x|x.timestamp_millis()).unwrap_or(cursor+3_600_000);
            let end=limit.min(boundary.max(cursor+1));
            buckets.entry((day,hour)).or_default().push((cursor,end,flagged));
            cursor=end;split_count+=1;
        }
    }
    let mut samples=Vec::new();
    for ((day,hour),spans) in buckets {
        let total=crate::policy::union_ms(spans.iter().map(|(a,b,_)|(*a,*b)).collect(),i64::MIN,i64::MAX);
        if total<MIN_HOUR_COVERAGE_MS {continue;}
        let distraction=spans.iter().any(|(a,b,flag)|*flag&&b>a);
        samples.push(Sample{day,hour,distraction});
    }
    Ok(score(&samples, now.hour()))
}

fn smoothed(success:usize,total:usize)->f64 { (success as f64 + 1.0) / (total as f64 + 2.0) }

fn score(data:&[Sample],hour:u32)->Value {
    let days=data.iter().map(|s|s.day).collect::<BTreeSet<_>>().len();
    let relevant=data.iter().filter(|s|s.hour==hour).collect::<Vec<_>>();
    if days<MIN_DAYS || relevant.len()<MIN_OBS_PER_HOUR {
        return json!({"available":false,"reason":format!("Not enough completed, observed history: {} days; {} qualifying observations at this hour. Requires at least {} days and {} observations for this hour.",days,relevant.len(),MIN_DAYS,MIN_OBS_PER_HOUR),"observed_days":days,"hour_samples":relevant.len()});
    }
    let global_wins=data.iter().filter(|s|s.distraction).count();
    let wins=relevant.iter().filter(|s|s.distraction).count();
    let prior=smoothed(global_wins,data.len());
    let mean=smoothed(wins,relevant.len());
    // Leave-one-out calibration. Compare against a base-rate-only prior computed
    // without the held-out sample. Low sample counts cannot earn high confidence.
    let mut error=0.0;let mut baseline=0.0;
    for item in &relevant {
        let y=if item.distraction{1.0}else{0.0};
        let p=smoothed(wins-item.distraction as usize,relevant.len()-1);
        let b=smoothed(global_wins-item.distraction as usize,data.len()-1);
        error+=(p-y)*(p-y);baseline+=(b-y)*(b-y);
    }
    error/=relevant.len() as f64; baseline/=relevant.len() as f64;
    if error>=baseline {
        return json!({"available":false,"reason":"This hour-specific pattern did not beat the historical base-rate baseline on held-out observations.","observed_days":days,"hour_samples":relevant.len(),"calibrated":false});
    }
    // Approximate Wilson interval for a binary occurrence with modest shrinkage.
    let n=relevant.len() as f64;let z=1.96;let raw=wins as f64/n;
    let center=(raw+z*z/(2.0*n))/(1.0+z*z/n);
    let radius=z*((raw*(1.0-raw)/n+z*z/(4.0*n*n)).sqrt())/(1.0+z*z/n);
    json!({"available":true,"hour_local":hour,"probability":mean,"interval_low":(center-radius).max(0.0),"interval_high":(center+radius).min(1.0),
        "confidence":if days>=35&&relevant.len()>=25{"moderate"}else{"low"},
        "observed_days":days,"hour_samples":relevant.len(),"out_of_sample_brier":error,"base_rate_brier":baseline,
        "signal":"Chance this local hour includes any activity in categories you marked as distracting, conditional on ≥5 minutes observed. This is only a local historical pattern, not certainty."})
}

#[cfg(test)]mod tests{
 use super::*;
 fn sample_days(count:i64,hour:u32,positive:bool)->Vec<Sample>{
  let origin=NaiveDate::from_ymd_opt(2026,1,1).unwrap();
  (0..count).map(|i|Sample{day:origin+chrono::Duration::days(i),hour,distraction:positive}).collect()
 }
 #[test]fn opt_in_cannot_bypass_minimum_observations(){assert!(!score(&sample_days(11,10,true),10)["available"].as_bool().unwrap());}
 #[test]fn constant_rate_does_not_claim_predictive_skill(){let x=sample_days(28,10,true);assert!(!score(&x,10)["available"].as_bool().unwrap());}
 #[test]fn distinctly_risky_hour_can_be_calibrated(){
  let mut all=sample_days(40,10,true);all.extend(sample_days(40,11,false));
  assert_eq!(score(&all,10)["available"],true);
 }
}

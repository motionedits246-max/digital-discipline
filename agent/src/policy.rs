//! Deterministic browser policy logic. No network or AI is used by enforcement.
use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, Timelike, TimeZone};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields, default)]
pub struct RuleConfig {
    pub action: String,             // block | hide_feed | stop_autoplay
    pub group_domains: Vec<String>, // additional sites in shared budget
    pub group_apps: Vec<String>, // executable basenames in shared budget
    pub group_categories: Vec<String>, // user-defined category matching
    pub excluded_domains: Vec<String>, // exceptions to usage accounting
    pub excluded_apps: Vec<String>,
    pub streak_gap_seconds: Option<u32>, // max gap between successive plays
    pub qualified_plays: Option<u32>,
    pub streak_plays: Option<u32>,
    pub streak_window_minutes: Option<u32>,
    pub include_replays: bool,
    pub weekdays: Vec<u32>,        // ISO Monday=1..Sunday=7. Empty => all days.
    pub start_time: Option<String>, // HH:MM local time
    pub end_time: Option<String>,
}
impl RuleConfig {
    pub fn validate(&self) -> bool {
        (self.action.is_empty() || ["block","hide_feed","stop_autoplay","notify"].contains(&self.action.as_str()))
            && self.group_domains.len()<=20 && self.excluded_domains.len()<=20
            && self.group_domains.iter().chain(self.excluded_domains.iter()).all(|s|crate::domain::valid_domain(s))
            && self.group_apps.len()<=30 && self.excluded_apps.len()<=30
            && self.group_apps.iter().chain(self.excluded_apps.iter()).all(|s|!s.is_empty() && s.len()<=100 && s.bytes().all(|c|c.is_ascii_alphanumeric()||b"-_.".contains(&c)))
            && self.group_categories.len()<=13
            && self.group_categories.iter().all(|s|["Development","Design","Productivity","Communication","Education","Entertainment","Gaming","Social","News","Utility","System","Browser","Unclassified"].contains(&s.as_str()))
            && self.streak_gap_seconds.is_none_or(|x|(10..=600).contains(&x))
            && self.qualified_plays.is_none_or(|x|x>=1 && x<=10000)
            && self.streak_plays.is_none_or(|x|x>=2 && x<=500)
            && self.streak_window_minutes.is_none_or(|x|x>=1 && x<=240)
            && self.streak_plays.is_some()==self.streak_window_minutes.is_some()
            && self.weekdays.len()<=7
            && (self.weekdays.is_empty() || self.start_time.is_some())
            && self.weekdays.iter().all(|x|(1..=7).contains(x))
            && match (&self.start_time,&self.end_time) {
                (None,None)=>true,
                (Some(a),Some(b))=>parse_hhmm(a).is_some() && parse_hhmm(b).is_some() && a!=b,
                _=>false,
            }
    }
    pub fn action(&self)->&str {if self.action.is_empty(){"block"}else{&self.action}}
}
pub fn parse_hhmm(value:&str)->Option<u32>{
    if value.len()!=5 || value.as_bytes()[2]!=b':' {return None;}
    let hours=value[..2].parse::<u32>().ok()?;
    let minutes=value[3..].parse::<u32>().ok()?;
    if hours>=24||minutes>=60 {None}else{Some(hours*60+minutes)}
}

/// Apply day-of-week to the **start** of an overnight window, not the current day.
/// No scheduled window means 24x7. DST uses the current local wall clock.
pub fn schedule_active(config:&RuleConfig, now: DateTime<Local>)->bool {
    let Some(start)=config.start_time.as_deref().and_then(parse_hhmm) else {return true};
    let Some(end)=config.end_time.as_deref().and_then(parse_hhmm) else {return false};
    let mins=now.hour()*60+now.minute();
    let start_day=if start>end && mins<end {now.date_naive()-Duration::days(1)} else {now.date_naive()};
    let weekday=start_day.weekday().number_from_monday();
    (config.weekdays.is_empty() || config.weekdays.contains(&weekday)) &&
        (if start<end {mins>=start && mins<end} else {mins>=start || mins<end})
}

/// Earliest local schedule boundary within eight days, not after the caller-supplied lease.
/// Ambiguous DST hours use the first matching instant, while skipped hours are simply missed.
pub fn next_schedule_boundary(config:&RuleConfig, now:DateTime<Local>)->Option<i64>{
    let start=config.start_time.as_deref().and_then(parse_hhmm)?;
    let end=config.end_time.as_deref().and_then(parse_hhmm)?;
    let date=now.date_naive();
    let mut boundaries=Vec::new();
    for i in -1..=8_i64 {
        let day=date+Duration::days(i);
        if !config.weekdays.is_empty() && !config.weekdays.contains(&day.weekday().number_from_monday()){continue;}
        let start_at=wall_time(day,start);
        let end_day=if start>end {day+Duration::days(1)}else{day};
        let end_at=wall_time(end_day,end);
        for b in [start_at,end_at].into_iter().flatten(){
            if b.timestamp_millis()>now.timestamp_millis() {boundaries.push(b.timestamp_millis());}
        }
    }
    boundaries.into_iter().min()
}
fn wall_time(day:NaiveDate, mins:u32)->Option<DateTime<Local>>{
    let naive=day.and_hms_opt(mins/60,mins%60,0)?;
    let possible=Local.from_local_datetime(&naive);
    possible.earliest().or_else(||possible.latest())
}

/// Union duration of clipped foreground intervals. Never sum overlapping observations.
pub fn union_ms(mut intervals:Vec<(i64,i64)>, start:i64,end:i64)->i64{
    if end<=start {return 0;}
    intervals=intervals.into_iter().map(|(a,b)|(a.max(start),b.min(end)))
        .filter(|(a,b)|b>a).collect();
    intervals.sort_unstable();
    let(mut total,mut active):(i64,Option<(i64,i64)>)=(0,None);
    for (a,b) in intervals {
        if let Some((lo,hi))=active {
            if a<=hi {active=Some((lo,hi.max(b)));}
            else {total+=hi-lo;active=Some((a,b));}
        } else {active=Some((a,b));}
    }
    if let Some((a,b))=active {total+=b-a;}
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn union_removes_overlap_and_clips(){assert_eq!(union_ms(vec![(0,200),(100,400),(650,800)],50,700),400);}
    #[test] fn schedule_input_validation(){
        assert_eq!(parse_hhmm("23:59"),Some(1439));assert_eq!(parse_hhmm("24:00"),None);
        assert!(!RuleConfig{start_time:Some("01:00".into()), ..Default::default()}.validate());
    }
    #[test] fn overnight_weekday_refers_to_previous_day(){
        let monday=chrono::NaiveDate::from_ymd_opt(2026,11,2).unwrap();
        let now=Local.from_local_datetime(&monday.and_hms_opt(23,0,0).unwrap()).earliest().unwrap();
        let tomorrow=now+Duration::hours(3);
        let c=RuleConfig{start_time:Some("22:00".into()),end_time:Some("06:00".into()),weekdays:vec![1], ..Default::default()};
        assert!(schedule_active(&c,now));assert!(schedule_active(&c,tomorrow));
        assert!(!schedule_active(&c,now+Duration::hours(10)));
    }
}

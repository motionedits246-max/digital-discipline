//! Exact hour attribution for observed foreground intervals, including sessions
//! crossing local hour boundaries and non-integer time zone offsets.
use chrono::{DateTime, Local, Timelike, Utc};

pub fn add_to_hours(bins:&mut [i64;24],start_ms:i64,end_ms:i64){
    let mut cursor=start_ms;
    let mut steps=0;
    while cursor<end_ms && steps<2000 {
        let Some(t)=DateTime::<Utc>::from_timestamp_millis(cursor) else {break;};
        let local=t.with_timezone(&Local);
        let hour=local.hour() as usize;
        let consumed=local.minute() as i64*60_000+local.second() as i64*1000+local.timestamp_subsec_millis() as i64;
        let remaining=3_600_000-consumed;
        let boundary=cursor.saturating_add(remaining.max(1));
        let until=end_ms.min(boundary);
        bins[hour]+=(until-cursor).max(0);
        cursor=until;
        steps+=1;
    }
}

#[cfg(test)]mod tests {
 use super::*;
 #[test]fn exact_hour_accounting_preserves_total(){
  let a=DateTime::parse_from_rfc3339("2026-01-01T09:50:00+05:30").unwrap().timestamp_millis();
  let b=a+20*60_000;
  let mut bins=[0;24];add_to_hours(&mut bins,a,b);
  assert_eq!(bins.iter().sum::<i64>(),20*60_000);
 }
}

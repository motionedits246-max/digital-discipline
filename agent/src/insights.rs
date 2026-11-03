//! Transparent heuristics: never label human attention as objectively wasted.
use serde::Serialize;

#[derive(Clone, Debug,Serialize)]
pub struct WeekComparison {pub current_ms:i64,pub previous_ms:i64,pub change_ms:i64,pub coverage_warning:bool}

pub fn longest_streak(mut timestamps:Vec<i64>,gap_ms:i64)->(u32,i64) {
    timestamps.sort_unstable();
    let(mut best,mut current,mut best_at)=(0u32,0u32,0);
    let mut last=None;
    for t in timestamps {
        if last.is_some_and(|p|t-p<=gap_ms && t>p){current+=1;}else{current=1;}
        if current>best {best=current;best_at=t;}
        last=Some(t);
    }
    (best,best_at)
}
pub fn count_switches(sessions:&[(String,i64,i64,bool)],start:i64,end:i64)->u64 {
    // Debounced transitions across contiguous observed foreground sessions only.
    // Long sleep/unknown/idle gaps break continuity instead of implying a switch.
    let mut last: Option<(&str,i64)>=None;let mut switches=0;
    for (app,a,b,idle) in sessions {
        if *b<start||*a>end {continue;}
        if *idle {last=None;continue;}
        if b-a<1500 {continue;} // ignore brief focus jitter
        if let Some((prev,prev_end))=last {
            if *a>=prev_end && *a-prev_end<=30_000 && prev!=app {switches+=1;}
        }
        last=Some((app,*b));
    }
    switches
}
#[cfg(test)]mod tests {
    use super::*;
    #[test]fn streak_doesnt_count_gap(){assert_eq!(longest_streak(vec![1000,5000,9000,800_000],15_000).0,3);}
    #[test]fn switches_ignore_unknown_gaps(){assert_eq!(count_switches(&[("a".into(),0,3000,false),("b".into(),120000,123000,false)],0,200000),0);}
    #[test]fn switches_ignore_idle_and_repetition(){assert_eq!(count_switches(&[("a".into(),0,2000,false),("a".into(),2000,5000,false),("b".into(),5000,8000,false)],0,10_000),1);}
}

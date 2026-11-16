//! Pure domain logic. These tests intentionally do not need Windows/DB/network.
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "1.0";
pub const POLL_GAP_MS: i64 = 10_000;
pub const PLAY_QUALIFY_MS: i64 = 2_000;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Observation {
    pub app: String,
    pub idle: bool,
    pub domain: Option<String>,
    pub profile_id: Option<String>,
    pub at_ms: i64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Segment {
    pub app: String,
    pub domain: Option<String>,
    pub profile_id: Option<String>,
    pub idle: bool,
    pub start_ms: i64,
    pub end_ms: i64,
}

#[derive(Default, Debug)]
pub struct Reconciler {
    current: Option<Segment>,
    high_water_ms: Option<i64>,
    pub clock_rollbacks: u64,
    pub observation_gaps: u64,
}

impl Reconciler {
    pub fn push(&mut self, obs: Observation) -> (Option<Segment>, Option<Segment>) {
        // Return a finalized segment and the current bounded heartbeat segment.
        // A tick gap never gets attributed to the old foreground app.
        if obs.at_ms < 0 { return (None, self.current.clone()); }
        // A wall-clock correction must never count the same UTC interval twice.
        // Wait for the previous watermark instead of assigning negative/replayed time.
        if self.high_water_ms.is_some_and(|last|obs.at_ms<last){
            if self.current.is_some(){self.clock_rollbacks+=1;}
            return (self.current.take(),None);
        }
        self.high_water_ms=Some(obs.at_ms);
        let previous = self.current.take();
        match previous {
            None => {
                self.current = Some(Segment { app: obs.app, domain: obs.domain, profile_id:obs.profile_id, idle: obs.idle, start_ms: obs.at_ms, end_ms: obs.at_ms });
                (None, self.current.clone())
            }
            Some(mut prev) => {
                let gap = obs.at_ms.saturating_sub(prev.end_ms);
                if gap>POLL_GAP_MS{self.observation_gaps+=1;}
                let same = prev.app == obs.app && prev.domain == obs.domain && prev.profile_id == obs.profile_id && prev.idle == obs.idle;
                if gap <= POLL_GAP_MS && same && gap >= 0 {
                    prev.end_ms = obs.at_ms;
                    self.current = Some(prev);
                    (None, self.current.clone())
                } else {
                    // The boundary is observed at the new sample, except across sleep/gaps.
                    if gap <= POLL_GAP_MS && gap >= 0 { prev.end_ms = obs.at_ms; }
                    let next = Segment { app: obs.app, domain: obs.domain, profile_id:obs.profile_id, idle: obs.idle, start_ms: obs.at_ms, end_ms: obs.at_ms };
                    self.current = Some(next);
                    (Some(prev), self.current.clone())
                }
            }
        }
    }
    pub fn stop(&mut self) -> Option<Segment> { self.current.take() }
}

pub fn valid_domain(domain: &str) -> bool {
    let s = domain.trim();
    if s.len() > 253 || !s.contains('.') || s.starts_with('.') || s.ends_with('.') || s.parse::<std::net::IpAddr>().is_ok() { return false; }
    s.split('.').all(|label| {
        !label.is_empty() && label.len() <= 63 && !label.starts_with('-') && !label.ends_with('-') &&
        label.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'-')
    }) && !s.chars().any(char::is_uppercase)
}

pub fn safe_path(path: &str) -> bool {
    path.starts_with('/') && path.len() <= 160 && !path.contains('\\') &&
    !path.contains('"') && !path.contains('\n') && !path.contains('\r') &&
    !path.contains('?') && !path.contains('#') && !path.contains("..") &&
    path.chars().all(|c| c.is_ascii_alphanumeric() || "/-_~.%".contains(c))
}

pub fn category(app: &str, domain: Option<&str>) -> &'static str {
    let a = app.to_ascii_lowercase();
    let d = domain.unwrap_or("");
    if ["code.exe", "cursor.exe", "antigravity.exe", "idea64.exe", "pycharm64.exe", "clion64.exe", "devenv.exe", "sublime_text.exe", "nvim.exe", "webstorm64.exe"].contains(&a.as_str()) { "Development" }
    else if ["github.com", "gitlab.com", "stackoverflow.com", "docs.rs"].contains(&d) { "Development" }
    else if ["youtube.com", "www.youtube.com", "instagram.com", "www.instagram.com"].contains(&d) { "Entertainment" }
    else if ["explorer.exe", "taskmgr.exe", "digital-discipline.exe"].contains(&a.as_str()) { "Utility" }
    else { "Unclassified" }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn obs(at: i64, app: &str, idle: bool) -> Observation {
        Observation { at_ms: at, app: app.into(), idle, domain: None,profile_id:None }
    }
    #[test]
    fn rollback_never_recounts_a_previously_observed_interval_even_after_pause(){
        let mut r=Reconciler::default();r.push(obs(1000,"code.exe",false));r.push(obs(5000,"code.exe",false));
        let (ended,current)=r.push(obs(2000,"brave.exe",false));assert_eq!(ended.unwrap().end_ms,5000);assert!(current.is_none());
        r.stop();assert!(r.push(obs(4000,"brave.exe",false)).1.is_none());
        let current=r.push(obs(6000,"brave.exe",false)).1.unwrap();assert_eq!(current.start_ms,6000);assert_eq!(r.clock_rollbacks,1);
    }
    #[test]
    fn switches_and_idle_cannot_overlap() {
        let mut r = Reconciler::default();
        r.push(obs(1000, "cursor.exe", false));
        r.push(obs(3000, "cursor.exe", false));
        let (a, _) = r.push(obs(5000, "brave.exe", false));
        let (b, _) = r.push(obs(7000, "brave.exe", true));
        let a = a.unwrap(); let b = b.unwrap();
        assert_eq!((a.start_ms, a.end_ms), (1000, 5000));
        assert_eq!((b.start_ms, b.end_ms), (5000, 7000));
        assert!(a.end_ms <= b.start_ms);
    }
    #[test]
    fn sleep_gap_is_not_counted() {
        let mut r = Reconciler::default();
        r.push(obs(1000, "code.exe", false));
        r.push(obs(3000, "code.exe", false));
        let (old, cur) = r.push(obs(3_603_000, "code.exe", false));
        assert_eq!(old.unwrap().end_ms, 3000);
        assert_eq!(cur.unwrap().start_ms, 3_603_000);
    }
    #[test]
    fn unknown_ide_is_not_falsely_development() {
        assert_eq!(category("custom-editor.exe", None), "Unclassified");
        assert_eq!(category("cursor.exe", None), "Development");
    }
    #[test]
    fn domains_and_paths_are_safe() {
        assert!(valid_domain("youtube.com"));
        assert!(!valid_domain("https://youtube.com"));
        assert!(!valid_domain("localhost"));
        assert!(!valid_domain("127.0.0.1"));
        assert!(safe_path("/shorts/"));
        assert!(!safe_path("/watch?token=123"));
    }
}

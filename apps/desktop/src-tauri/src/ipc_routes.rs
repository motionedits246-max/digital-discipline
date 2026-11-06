//! Pure route validation before any UTF-8 byte slicing or privileged IPC.
pub fn allowed(method:&str,path:&str)->bool {
    if !path.is_ascii()||path.len()>512{return false;}
    match (method,path) {
        ("GET","/state"|"/privacy/session"|"/live"|"/reports/heatmap"|"/export"|"/videos"|"/insights"|"/categories"|"/goals"|"/workflows"|"/health/details"|"/tracking/doctor"|"/settings"|"/classifications"|"/devices"|"/devices/export"|"/rules/audit"|"/browser-pairs"|"/coach/credentials") => true,
        ("GET",p) if p.starts_with("/sessions/") && p.len()==20 && p[10..].bytes().all(|c|c.is_ascii_digit()||c==b'-') => true,
        ("POST","/privacy/session"|"/data/purge"|"/tracking/pause"|"/tracking/doctor/rendered"|"/tracking/profiles/access"|"/rules"|"/categories/override"|"/goals"|"/workflows/profiles"|"/workflows/tasks"|"/focus/start"|"/focus/end"|"/focus/hint/choice"|"/settings"|"/coach"|"/coach/credentials"|"/classifications/config"|"/classifications/preferences"|"/devices/import"|"/rules/preview"|"/browser-pairs") => true,
        ("POST",p) if p.starts_with("/workflows/tasks/") && p.len()==53 && p[17..].bytes().all(|c|c.is_ascii_hexdigit()||c==b'-') => true,
        ("DELETE",p) if p.starts_with("/workflows/profiles/") && p.len()==56 && p[20..].bytes().all(|c|c.is_ascii_hexdigit()||c==b'-') => true,
        ("DELETE","/data") => true,
        ("DELETE","/classifications/config") => true,
        ("DELETE",p) if matches!(p,"/coach/credentials/openai"|"/coach/credentials/gemini"|"/coach/credentials/anthropic"|"/coach/credentials/groq"|"/coach/credentials/openrouter") => true,
        ("DELETE",p) if p.starts_with("/browser-pairs/") && p.len()==47 && p[15..].bytes().all(|b|(b'a'..=b'p').contains(&b)) => true,
        ("DELETE",p) if (p.starts_with("/rules/") || p.starts_with("/goals/") || p.starts_with("/devices/")) &&
            p.rsplit('/').next().is_some_and(|id|id.len()==36 && id.chars().all(|c|c.is_ascii_hexdigit()||c=='-')) => true,
        ("DELETE",p) if p.starts_with("/categories/override/") && p.len()<300 &&
            p[21..].bytes().all(|c|c.is_ascii_alphanumeric()||b"-_.:".contains(&c)) => true,
        _ => false,
    }
}

#[cfg(test)]mod tests{
 use super::*;
 #[test]fn untrusted_unicode_and_proxy_routes_are_rejected_without_panicking(){
  for p in ["/sessions/ééééé", "/categories/override/界", "/workflows/tasks/é", "/shutdown", "https://example.com", "/state?x=1", "//state"]{assert!(!allowed("GET",p));assert!(!allowed("DELETE",p));}
  assert!(allowed("GET","/sessions/2026-11-06"));assert!(allowed("GET","/tracking/doctor"));assert!(!allowed("POST","/state"));
 }
}

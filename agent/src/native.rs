//! Chromium native-messaging stdio host. Must print only framed protocol data to stdout.
use std::io::{self, Read, Write};
use serde_json::Value;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(windows)] unsafe {
        // Native messaging framing must bypass Windows CRT CRLF translation.
        // _O_BINARY = 0x8000 in the Universal C Runtime.
        #[link(name = "ucrt")]
        extern "C" { fn _setmode(fd: i32, mode: i32) -> i32; }
        _setmode(0, 0x8000);
        _setmode(1, 0x8000);
    }
    // Return a framed, actionable error instead of exiting with no reply when the agent token is missing.
    let token = crate::read_token().ok();
    let client = reqwest::blocking::Client::builder().timeout(std::time::Duration::from_secs(3)).build()?;
    let mut input = io::stdin().lock();
    let mut output = io::stdout().lock();
    loop {
        let mut size = [0u8; 4];
        match input.read_exact(&mut size) {
            Ok(_) => {},
            Err(ref e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        }
        let len = u32::from_le_bytes(size) as usize;
        if len == 0 || len > 65_536 { return Err("native message too large".into()); }
        let mut bytes = vec![0u8; len];
        input.read_exact(&mut bytes)?;
        let message: Value = serde_json::from_slice(&bytes)?;
        let op = message.get("op").and_then(Value::as_str).unwrap_or("");
        let (method, path) = match op {
            "event" => ("POST", "/events"),
            "sync" => ("GET", "/extension/rules"),
            _ => { write_frame(&mut output, &serde_json::json!({"ok":false,"error":"unsupported_op"}))?; continue; }
        };
        let response = if let Some(token) = token.as_ref() {
            let url = format!("http://127.0.0.1:48761{}", path);
            let req = client.request(method.parse()?, url).bearer_auth(token);
            let req = if method == "POST" { req.json(message.get("event").unwrap_or(&Value::Null)) } else { req };
            match req.send() {
                Ok(r) => {
                    let status = r.status();
                    if status.is_success() {
                        r.json::<Value>().unwrap_or(serde_json::json!({"ok":false,"error":"agent_invalid_response"}))
                    } else {serde_json::json!({"ok":false,"error":format!("agent_http_{}",status.as_u16())})}
                },
                Err(_) => serde_json::json!({"ok":false,"error":"agent_unavailable"}),
            }
        } else {serde_json::json!({"ok":false,"error":"agent_token_missing"})};
        write_frame(&mut output, &response)?;
    }
    Ok(())
}
fn write_frame(writer: &mut impl Write, value: &Value) -> io::Result<()> {
    let payload = serde_json::to_vec(value).map_err(io::Error::other)?;
    if payload.len() > 1_048_576 { return Err(io::Error::new(io::ErrorKind::InvalidData, "response too large")); }
    writer.write_all(&(payload.len() as u32).to_le_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()
}

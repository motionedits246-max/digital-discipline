use serde_json::{json, Value};
use std::sync::Mutex;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

static PREVIOUS: Mutex<Option<(Instant, u64)>> = Mutex::new(None);

/// Sample the current process. One busy core represents 100% CPU.
pub fn sample(scope: &str) -> Value {
    let mut previous = PREVIOUS.lock().unwrap_or_else(|error| error.into_inner());
    let Some((cpu, memory)) = process_times_and_memory() else {
        return json!({"available":false,"scope":scope});
    };
    let now = Instant::now();
    let percent = previous.and_then(|(at, prior)| {
        let elapsed = now.duration_since(at).as_secs_f64();
        (elapsed >= 0.1 && cpu >= prior).then(|| (cpu - prior) as f64 / 100_000.0 / elapsed)
    });
    if previous.is_none_or(|(at, _)| now.duration_since(at).as_millis() >= 100) {
        *previous = Some((now, cpu));
    }
    let sampled_at_ms = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis() as u64;
    json!({
        "available":true,"working_set_bytes":memory,"cpu_percent_one_core_basis":percent,
        "sampled_at_ms":sampled_at_ms,"scope":scope
    })
}

#[cfg(windows)]
fn process_times_and_memory() -> Option<(u64, u64)> {
    use windows_sys::Win32::{
        Foundation::FILETIME,
        System::{
            ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS},
            Threading::{GetCurrentProcess, GetProcessTimes},
        },
    };
    unsafe {
        let handle = GetCurrentProcess();
        let mut created: FILETIME = std::mem::zeroed();
        let mut ended: FILETIME = std::mem::zeroed();
        let mut kernel: FILETIME = std::mem::zeroed();
        let mut user: FILETIME = std::mem::zeroed();
        if GetProcessTimes(handle, &mut created, &mut ended, &mut kernel, &mut user) == 0 {
            return None;
        }
        let size = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        let mut memory: PROCESS_MEMORY_COUNTERS = std::mem::zeroed();
        memory.cb = size;
        if GetProcessMemoryInfo(handle, &mut memory, size) == 0 {
            return None;
        }
        let ticks = |time: FILETIME| (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
        Some((ticks(kernel).saturating_add(ticks(user)), memory.WorkingSetSize as u64))
    }
}

#[cfg(not(windows))]
fn process_times_and_memory() -> Option<(u64, u64)> {
    None
}

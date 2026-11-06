use crate::domain::Observation;

#[cfg(windows)]
pub fn observe(at_ms: i64, idle_limit_ms: u32) -> Option<Observation> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows_sys::Win32::System::SystemInformation::GetTickCount;
    use windows_sys::Win32::System::Threading::{OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION};
    use windows_sys::Win32::Foundation::CloseHandle;
    unsafe {
        let hwnd = GetForegroundWindow();
        if (hwnd as usize) == 0 { return None; }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == 0 { return None; }
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if (handle as usize) == 0 { return None; }
        let mut buffer = [0u16; 1024];
        let mut size = buffer.len() as u32;
        let ok = QueryFullProcessImageNameW(handle, 0, buffer.as_mut_ptr(), &mut size);
        CloseHandle(handle);
        if ok == 0 { return None; }
        let path = String::from_utf16_lossy(&buffer[..size as usize]);
        // Persist the executable basename ONLY: paths and window titles are private.
        let app = path.rsplit(['\\', '/']).next().unwrap_or("unknown").to_ascii_lowercase();
        // Lock-screen/signon shells are not ordinary user activity.
        if ["lockapp.exe", "logonui.exe"].contains(&app.as_str()) { return None; }
        let mut last = LASTINPUTINFO { cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32, dwTime: 0 };
        let idle = GetLastInputInfo(&mut last) != 0 && GetTickCount().wrapping_sub(last.dwTime) > idle_limit_ms;
        Some(Observation { app, domain: None, profile_id:None, idle, at_ms })
    }
}

#[cfg(not(windows))]
pub fn observe(_at_ms: i64, _idle_limit_ms: u32) -> Option<Observation> { None }

#[cfg(windows)]
pub fn process_resources()->Option<(u64,u64)>{
 use windows_sys::Win32::{System::{Threading::{GetCurrentProcess,GetProcessTimes},ProcessStatus::{GetProcessMemoryInfo,PROCESS_MEMORY_COUNTERS}},Foundation::FILETIME};
 unsafe{
  let handle=GetCurrentProcess();let mut created:FILETIME=std::mem::zeroed();let mut ended:FILETIME=std::mem::zeroed();let mut kernel:FILETIME=std::mem::zeroed();let mut user:FILETIME=std::mem::zeroed();
  if GetProcessTimes(handle,&mut created,&mut ended,&mut kernel,&mut user)==0{return None;}
  let mut memory:PROCESS_MEMORY_COUNTERS=std::mem::zeroed();memory.cb=std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
  let memory_size=memory.cb;
  if GetProcessMemoryInfo(handle,&mut memory,memory_size)==0{return None;}
  let ticks=|t:FILETIME|((t.dwHighDateTime as u64)<<32)|t.dwLowDateTime as u64;
  Some((ticks(kernel).saturating_add(ticks(user)),memory.WorkingSetSize as u64))
 }
}
#[cfg(not(windows))]pub fn process_resources()->Option<(u64,u64)>{None}

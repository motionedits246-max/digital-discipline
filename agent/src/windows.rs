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

#[cfg(not(windows))]pub fn application_description(_app:&str)->String{String::new()}
#[cfg(windows)]pub fn application_description(app:&str)->String{
 use windows_sys::Win32::{UI::WindowsAndMessaging::{GetForegroundWindow,GetWindowThreadProcessId},System::Threading::{OpenProcess,QueryFullProcessImageNameW,PROCESS_QUERY_LIMITED_INFORMATION},Foundation::CloseHandle};
 use std::ffi::c_void;
 #[link(name="version")]extern "system"{fn GetFileVersionInfoSizeW(file:*const u16,handle:*mut u32)->u32;fn GetFileVersionInfoW(file:*const u16,handle:u32,len:u32,data:*mut c_void)->i32;fn VerQueryValueW(block:*const c_void,query:*const u16,buffer:*mut *mut c_void,len:*mut u32)->i32;}
 #[link(name="kernel32")]extern "system"{fn GetDriveTypeW(root:*const u16)->u32;}
 unsafe{
  let mut pid=0;GetWindowThreadProcessId(GetForegroundWindow(),&mut pid);if pid==0{return String::new();}let handle=OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,0,pid);if handle as usize==0{return String::new();}
  let mut path=[0u16;2048];let mut len=path.len() as u32;let ok=QueryFullProcessImageNameW(handle,0,path.as_mut_ptr(),&mut len);CloseHandle(handle);if ok==0||len as usize>=path.len(){return String::new();}
  let text=String::from_utf16_lossy(&path[..len as usize]);if !text.rsplit(['\\','/']).next().is_some_and(|name|name.eq_ignore_ascii_case(app))||path[1]!=b':' as u16{return String::new();}
  let drive=[path[0],b':' as u16,b'\\' as u16,0];if GetDriveTypeW(drive.as_ptr())!=3{return String::new();}path[len as usize]=0;
  let mut unused=0;let size=GetFileVersionInfoSizeW(path.as_ptr(),&mut unused);if size==0||size>512000{return String::new();}let mut block=vec![0u8;size as usize];if GetFileVersionInfoW(path.as_ptr(),0,size,block.as_mut_ptr().cast())==0{return String::new();}
  let mut translation=std::ptr::null_mut();let mut count=0;let query="\\VarFileInfo\\Translation\0".encode_utf16().collect::<Vec<_>>();let mut pair=(0x0409u16,0x04b0u16);
  if VerQueryValueW(block.as_ptr().cast(),query.as_ptr(),&mut translation,&mut count)!=0&&count>=4&&!translation.is_null(){pair=(std::ptr::read_unaligned(translation.cast::<u16>()),std::ptr::read_unaligned(translation.cast::<u16>().add(1)));}
  let mut descriptions=Vec::new();for name in ["FileDescription","ProductName"]{let query=format!("\\StringFileInfo\\{:04x}{:04x}\\{name}\0",pair.0,pair.1).encode_utf16().collect::<Vec<_>>();let mut value=std::ptr::null_mut();let mut count=0;
   if VerQueryValueW(block.as_ptr().cast(),query.as_ptr(),&mut value,&mut count)!=0&&!value.is_null()&&count>1{let utf16=(0..(count as usize).min(180)).map(|i|std::ptr::read_unaligned(value.cast::<u16>().add(i))).take_while(|c|*c!=0).collect::<Vec<_>>();descriptions.push(String::from_utf16_lossy(&utf16));}
  }crate::service_discovery::clean_description(&descriptions.join(" · "))
 }
}

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

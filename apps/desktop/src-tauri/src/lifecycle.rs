//! Per-session desktop ownership and a bounded supervisor for our authenticated agent.
use std::{ffi::c_void,fs,path::PathBuf,sync::{Arc,Mutex,atomic::{AtomicBool,Ordering}},thread,time::{Duration,Instant},net::{TcpStream,SocketAddr},os::windows::process::CommandExt,process::Command};
use serde_json::{json,Value};
use tauri::{Manager,Emitter};
#[link(name="kernel32")] extern "system"{
 fn CreateMutexW(attributes:*const c_void,owner:i32,name:*const u16)->*mut c_void;
 fn CreateEventW(attributes:*const c_void,manual:i32,initial:i32,name:*const u16)->*mut c_void;
 fn GetLastError()->u32;fn SetEvent(handle:*mut c_void)->i32;
 fn WaitForSingleObject(handle:*mut c_void,milliseconds:u32)->u32;
 fn CloseHandle(handle:*mut c_void)->i32;
}
fn wide(s:&str)->Vec<u16>{s.encode_utf16().chain(Some(0)).collect()}
pub struct SingleInstance{lock:isize,event:isize}
impl SingleInstance{
 pub fn acquire()->Result<Option<Self>,String>{unsafe{
  let mutex=CreateMutexW(std::ptr::null(),0,wide("Local\\DigitalDiscipline.Desktop.v1").as_ptr());
  let exists=GetLastError()==183;
  if mutex.is_null(){return Err("Desktop ownership unavailable".into());}
  let event=CreateEventW(std::ptr::null(),0,0,wide("Local\\DigitalDiscipline.ShowWorkspace.v1").as_ptr());
  if event.is_null(){CloseHandle(mutex);return Err("Desktop activation unavailable".into());}
  if exists{SetEvent(event);CloseHandle(event);CloseHandle(mutex);return Ok(None);}
  Ok(Some(Self{lock:mutex as isize,event:event as isize}))
 }}
 pub fn event(&self)->isize{self.event}
}
impl Drop for SingleInstance{fn drop(&mut self){unsafe{CloseHandle(self.event as *mut c_void);CloseHandle(self.lock as *mut c_void);}}}
#[derive(Clone)]pub struct Supervisor{pub privacy_mask:Arc<AtomicBool>,wanted:Arc<AtomicBool>,state:Arc<Mutex<Value>>}
impl Supervisor{
 pub fn new()->Self{Self{privacy_mask:Arc::new(AtomicBool::new(false)),wanted:Arc::new(AtomicBool::new(true)),state:Arc::new(Mutex::new(json!({"state":"starting","restart_count":0})))}}
 pub fn wanted(&self)->bool{self.wanted.load(Ordering::SeqCst)}
 pub fn stop(&self){self.wanted.store(false,Ordering::SeqCst);}
 pub fn status(&self)->Value{self.state.lock().unwrap().clone()}
 fn update(&self,status:&str,count:u32){*self.state.lock().unwrap()=json!({"state":status,"restart_count":count,"expected_version":env!("CARGO_PKG_VERSION"),"wanted":self.wanted.load(Ordering::SeqCst)});}
}
fn occupied()->bool{TcpStream::connect_timeout(&SocketAddr::from(([127,0,0,1],48761)),Duration::from_millis(350)).is_ok()}
fn health(client:&reqwest::blocking::Client,token:&str)->Option<Value>{
 let response=client.get("http://127.0.0.1:48761/health").bearer_auth(token).send().ok()?;
 if !response.status().is_success(){return None;}
 let value:Value=response.json().ok()?;
 // The older agent had no application/version fields, but required the same local token.
 (value["ok"]==true&&value["schema_version"]=="1.0").then_some(value)
}
pub fn start(app:&tauri::AppHandle,supervisor:&Supervisor,event:isize){
 let activation=app.clone();let active=supervisor.clone();
 thread::spawn(move||while active.wanted.load(Ordering::SeqCst){
  if unsafe{WaitForSingleObject(event as *mut c_void,500)}==0{
   if let Some(w)=activation.get_webview_window("main"){let _=w.show();let _=crate::set_presentation(activation.clone(),"full".into());let _=w.set_focus();let _=activation.emit("dd:open-dashboard",());}
  }
 });
 let exe=app.path().resource_dir().ok().map(|d|d.join("bin/digital-discipline-agent.exe"));
 let monitor=supervisor.clone();
 thread::spawn(move||{
  let Some(exe)=exe.filter(|p|p.is_file()) else{monitor.update("bundled_agent_missing",0);return;};
  let Ok(client)=reqwest::blocking::Client::builder().timeout(Duration::from_secs(2)).redirect(reqwest::redirect::Policy::none()).build() else{return;};
  let mut launches=std::collections::VecDeque::<Instant>::new();let mut count=0u32;let mut failures=0u32;let mut next=Instant::now();
  while monitor.wanted.load(Ordering::SeqCst){
   let token=crate::token_path().ok().and_then(|p|fs::read_to_string(p).ok()).unwrap_or_default();
   match health(&client,token.trim()){
    Some(info) if info["version"]==env!("CARGO_PKG_VERSION")=>{monitor.update("healthy",count);{let mut status=monitor.state.lock().unwrap();status["tracking_paused"]=info["tracking_paused"].clone();status["private_session"]=info["private_session"].clone();}failures=0;},
    Some(_)=>{
     monitor.update("replacing_previous_agent",count);
     let _=client.post("http://127.0.0.1:48761/shutdown").bearer_auth(token.trim()).json(&json!({})).send();
     for _ in 0..50{if !occupied()||!monitor.wanted.load(Ordering::SeqCst){break;}thread::sleep(Duration::from_millis(100));}
     if occupied(){monitor.update("previous_agent_did_not_stop",count);}
    },
    None=>{if occupied(){monitor.update("port_in_use_or_auth_failed",count);}}
   }
   let now=Instant::now();while launches.front().is_some_and(|at|now.duration_since(*at)>Duration::from_secs(600)){launches.pop_front();}
   if monitor.wanted.load(Ordering::SeqCst)&&!occupied()&&now>=next{
    if launches.len()>=5{monitor.update("restart_limit_reached",count);}else{
     launches.push_back(now);failures=failures.saturating_add(1);next=now+Duration::from_secs((5u64<<failures.min(4)).min(60));
     match launch(&exe){Ok(())=>{count+=1;monitor.update("starting",count);},Err(_)=>monitor.update("agent_launch_failed",count)}
    }
   }
   for _ in 0..10{if !monitor.wanted.load(Ordering::SeqCst){break;}thread::sleep(Duration::from_millis(500));}
  }
 });
}
fn launch(exe:&PathBuf)->std::io::Result<()>{Command::new(exe).creation_flags(0x08000000).spawn().map(|_|())}

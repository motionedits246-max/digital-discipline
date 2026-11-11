#![cfg_attr(all(windows,not(debug_assertions)), windows_subsystem = "windows")]
mod ipc_routes;
#[path = "../../../../shared/process_resources.rs"]
mod process_resources;
mod window_policy;
#[cfg(windows)] mod lifecycle;
use serde_json::Value;
use std::{fs, path::PathBuf, time::Duration};
#[cfg(windows)] use std::io::{Read,Write};
#[cfg(windows)] use std::{net::{TcpStream,SocketAddr},process::Command};
use tauri::{Manager,Emitter, LogicalSize, Size};

#[derive(Default)]struct PresentationState{current:std::sync::Mutex<(String,Option<(i32,i32,u32,u32,u64)>)>}
fn monitor_key(m:&tauri::window::Monitor)->(i32,i32,u32,u32,u64){let a=m.work_area();(a.position.x,a.position.y,a.size.width,a.size.height,m.scale_factor().to_bits())}
#[derive(Default)]struct ServiceHintState{payload:std::sync::Mutex<Option<Value>>}
fn hints_masked(app:&tauri::AppHandle)->bool{
 #[cfg(windows)]{return app.try_state::<lifecycle::Supervisor>().is_some_and(|s|{let status=s.status();s.privacy_mask.load(std::sync::atomic::Ordering::SeqCst)||status["private_session"]["active"]==true||status["tracking_paused"]==true});}
 #[cfg(not(windows))]{let _=app;false}
}
async fn show_notice(app:tauri::AppHandle,notice:Value)->Result<bool,String>{
 if hints_masked(&app){return Ok(false);}
 {let state=app.state::<ServiceHintState>();let mut payload=state.payload.lock().map_err(|_|"Notification unavailable")?;if payload.is_some(){return Ok(false);}*payload=Some(notice.clone());}
 if app.get_webview_window("service-hint").is_none(){
  if tauri::WebviewWindowBuilder::new(&app,"service-hint",tauri::WebviewUrl::App("index.html?surface=service-hint".into())).title("Digital Discipline notification").inner_size(356.0,156.0).resizable(false).decorations(false).transparent(true).shadow(false).always_on_top(true).skip_taskbar(true).focused(false).visible(false).build().is_err(){
   *app.state::<ServiceHintState>().payload.lock().map_err(|_|"Notification unavailable")?=None;return Err("Could not open notification".into());
  }
 }else{app.emit_to("service-hint","dd:service-hint",notice).map_err(|_|"Could not deliver notification")?;}Ok(true)
}
#[tauri::command]async fn deliver_service_hint(app:tauri::AppHandle,service:Value)->Result<bool,String>{
 if service["subject"].as_str().is_none_or(|s|s.len()>260)||serde_json::to_vec(&service).map_err(|_|"Invalid suggestion")?.len()>4096{return Err("Invalid suggestion".into());}
 show_notice(app,serde_json::json!({"kind":"service","service":service})).await
}
#[tauri::command]async fn deliver_desktop_notice(app:tauri::AppHandle,notice:Value)->Result<bool,String>{
 if serde_json::to_vec(&notice).map_err(|_|"Invalid notification")?.len()>4096||!matches!(notice["kind"].as_str(),Some("browser"|"status"))||!matches!(notice["page"].as_str(),Some("Settings & Privacy"|"Inbox"))||notice["title"].as_str().is_none_or(|s|s.is_empty()||s.len()>120)||notice["detail"].as_str().is_none_or(|s|s.len()>400){return Err("Invalid notification".into());}
 show_notice(app,notice).await
}
#[tauri::command]fn read_service_hint(app:tauri::AppHandle)->Option<Value>{if hints_masked(&app){None}else{app.state::<ServiceHintState>().payload.lock().ok().and_then(|v|v.clone())}}
#[tauri::command]fn set_service_hint_window(app:tauri::AppHandle,visible:bool,expanded:bool)->Result<(),String>{
 if !visible||hints_masked(&app){
  *app.state::<ServiceHintState>().payload.lock().map_err(|_|"Notification unavailable")?=None;
  if let Some(w)=app.get_webview_window("service-hint"){let _=w.hide();return w.close().map_err(|_|"Could not close notification".into());}return Ok(());
 }
 if app.state::<ServiceHintState>().payload.lock().map_err(|_|"Notification unavailable")?.is_none(){return Ok(());}
 let Some(w)=app.get_webview_window("service-hint")else{return Ok(());};
 let monitor=app.get_webview_window("main").and_then(|main|main.current_monitor().ok().flatten()).or_else(||w.primary_monitor().ok().flatten());
 let (width,height)=if let Some(ref m)=monitor{(356.0_f64.min(m.work_area().size.width as f64/m.scale_factor()-16.0),f64::min(if expanded{420.0}else{156.0},m.work_area().size.height as f64/m.scale_factor()-16.0))}else{(356.0,if expanded{420.0}else{156.0})};
 w.set_size(Size::Logical(LogicalSize::new(width.max(1.0),height.max(1.0)))).map_err(|_|"Could not size notification")?;
 if let Some(m)=monitor{let scale=m.scale_factor();let size=w.outer_size().map_err(|_|"Notification size unavailable")?;let bounds=m.work_area();let margin=(8.0*scale) as i32;let x=bounds.position.x+(bounds.size.width as i32-size.width as i32-margin).max(0);let y=bounds.position.y+(bounds.size.height as i32-size.height as i32-margin).max(0);w.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x,y))).map_err(|_|"Could not position notification")?;}
 #[cfg(windows)]{
  #[link(name="user32")]extern "system"{fn ShowWindow(hwnd:*mut std::ffi::c_void,command:i32)->i32;}
  let hwnd=w.hwnd().map_err(|_|"Notification handle unavailable")?;unsafe{ShowWindow(hwnd.0 as *mut std::ffi::c_void,4);}// Do not steal focus from the foreground application.
  Ok(())
 }
 #[cfg(not(windows))]{w.show().map_err(|_|"Could not show notification".into())}
}
#[tauri::command]fn open_service_review(app:tauri::AppHandle)->Result<(),String>{let _=set_service_hint_window(app.clone(),false,false);set_presentation(app.clone(),"full".into())?;if let Some(w)=app.get_webview_window("main"){let _=w.show();let _=w.set_focus();}app.emit_to("main","dd:open-service-review",()).map_err(|_|"Could not open service review".into())}
#[tauri::command]fn open_notice_page(app:tauri::AppHandle,page:String)->Result<(),String>{
 if !matches!(page.as_str(),"Settings & Privacy"|"Inbox"){return Err("Unsupported notification destination".into());}
 set_service_hint_window(app.clone(),false,false)?;set_presentation(app.clone(),"full".into())?;
 if let Some(w)=app.get_webview_window("main"){w.show().map_err(|e|e.to_string())?;w.set_focus().map_err(|e|e.to_string())?;}
 app.emit_to("main","dd:open-notice-page",page).map_err(|_|"Could not open notification details".into())
}

fn token_path() -> Result<PathBuf,String> {
    let dir=std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA unavailable")?;
    Ok(PathBuf::from(dir).join("DigitalDiscipline").join("agent.token"))
}

#[tauri::command]
async fn agent_request(method:String,path:String,body:Option<Value>) -> Result<Value,String> {
    // Strict IPC allowlist prevents a compromised WebView from becoming an arbitrary HTTP proxy.
    let allowed=ipc_routes::allowed(&method,&path);
    if !allowed { return Err("Command or route not allowed".into()); }
    let token=fs::read_to_string(token_path()?).map_err(|_|"Agent not started; token unavailable".to_owned())?;
    let client=reqwest::Client::builder().timeout(Duration::from_secs(if path=="/coach" {58}else if path.starts_with("/backup")||path.starts_with("/sync")||path.starts_with("/updates/"){180}else{6})).build().map_err(|e|e.to_string())?;
    let mut request=client.request(method.parse().map_err(|_|"Invalid HTTP method")?,format!("http://127.0.0.1:48761{}",path)).bearer_auth(token.trim());
    if method=="POST" {request=request.json(&body.unwrap_or(Value::Null));}
    let response=request.send().await.map_err(|_|"Local agent unavailable".to_owned())?;
    if !response.status().is_success() {
        let status=response.status();
        let detail=response.text().await.unwrap_or_default();
        if detail.len()<=100 && detail.bytes().all(|byte|byte.is_ascii_alphanumeric()||byte==b'_') {
            return Err(format!("Agent reported {}: {}",status,detail.replace('_'," ")));
        }
        return Err(format!("Agent reported {}",status));
    }
    response.json::<Value>().await.map_err(|_|"Invalid agent response".into())
}

#[tauri::command]
fn set_presentation(app:tauri::AppHandle,mode:String)->Result<(),String> {
    let w=app.get_webview_window("main").ok_or("Window unavailable")?;
    let monitor=w.current_monitor().map_err(|e|e.to_string())?.or(w.primary_monitor().map_err(|e|e.to_string())?);
    let (work_width,work_height)=monitor.as_ref().map(|m|(m.work_area().size.width as f64/m.scale_factor(),m.work_area().size.height as f64/m.scale_factor())).unwrap_or((1920.0,1080.0));
    let bounds=window_policy::layout(&mode,work_width,work_height).ok_or("Unsupported mode")?;
    *app.state::<PresentationState>().current.lock().map_err(|_|"Window mode unavailable")?=(mode.clone(),monitor.as_ref().map(monitor_key));
    let popout=mode=="popout";
    // Clear the previous mode's floor before shrinking back to the controller.
    w.set_min_size(None::<Size>).map_err(|e|e.to_string())?;
    w.unmaximize().map_err(|e|e.to_string())?;
    w.set_resizable(bounds.workspace).map_err(|e|e.to_string())?;
    w.set_size(Size::Logical(LogicalSize::new(bounds.width,bounds.height))).map_err(|e|e.to_string())?;
    w.set_min_size(Some(Size::Logical(LogicalSize::new(bounds.min_width,bounds.min_height)))).map_err(|e|e.to_string())?;
    w.set_always_on_top(!popout).map_err(|e|e.to_string())?;
    w.set_skip_taskbar(!popout).map_err(|e|e.to_string())?;
    w.set_decorations(popout).map_err(|e|e.to_string())?;
    if popout {w.maximize().map_err(|e|e.to_string())?;}
    // Keep the user's chosen position when switching between pill, panel, and workspace.
    // Clamp after resizing so a larger panel cannot disappear beyond a monitor edge.
    if let Some(monitor)=w.current_monitor().map_err(|e|e.to_string())? {
        let current=w.outer_position().map_err(|e|e.to_string())?;
        let size=w.outer_size().map_err(|e|e.to_string())?;
        let bounds=monitor.work_area();
        let left=bounds.position.x+8;
        let top=bounds.position.y+8;
        let right=(bounds.position.x+bounds.size.width as i32-size.width as i32-8).max(left);
        let bottom=(bounds.position.y+bounds.size.height as i32-size.height as i32-8).max(top);
        let x=current.x.clamp(left,right);
        let y=current.y.clamp(top,bottom);
        if x!=current.x||y!=current.y {
            w.set_position(tauri::Position::Physical(tauri::PhysicalPosition::new(x,y))).map_err(|e|e.to_string())?;
        }
    }
    Ok(())
}
#[tauri::command]
fn drag_window(app:tauri::AppHandle)->Result<(),String>{
    let w=app.get_webview_window("main").ok_or("Window unavailable")?;
    w.start_dragging().map_err(|e|e.to_string())
}
#[tauri::command]
fn hide_to_tray(app:tauri::AppHandle)->Result<(),String>{
    let w=app.get_webview_window("main").ok_or("Window unavailable")?;
    w.hide().map_err(|e|e.to_string())
}

#[tauri::command]
fn pair_browser(app:tauri::AppHandle,browser:String,extension_id:String)->Result<String,String>{
    #[cfg(not(windows))] {let _=(app,browser,extension_id);return Err("Browser pairing is supported on Windows only".into());}
    #[cfg(windows)] {
        if !["Brave","Chrome","Edge"].contains(&browser.as_str())||extension_id.len()!=32||!extension_id.bytes().all(|b|(b'a'..=b'p').contains(&b)){
            return Err("Invalid Chromium extension ID or browser".into());
        }
        let exe=app.path().resource_dir().map_err(|e|e.to_string())?.join("bin").join("digital-discipline-agent.exe");
        if !exe.exists(){return Err("Bundled agent unavailable; install the application first".into());}
        let base=std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA unavailable")?;
        let data=PathBuf::from(base).join("DigitalDiscipline");
        fs::create_dir_all(&data).map_err(|e|e.to_string())?;
        let manifest=data.join(format!("com.digitaldiscipline.bridge.{browser}.json"));
        let current:Vec<String>=fs::read(&manifest).ok()
            .and_then(|bytes|serde_json::from_slice::<Value>(&bytes).ok())
            .and_then(|value|value.get("allowed_origins").and_then(Value::as_array).cloned())
            .unwrap_or_default().into_iter().filter_map(|v|v.as_str().map(str::to_string))
            .filter(|s|s.len()==52 && s.starts_with("chrome-extension://") && s.ends_with('/') &&
                s[19..51].bytes().all(|b|(b'a'..=b'p').contains(&b))).take(15).collect();
        let mut allowed=current;
        let origin=format!("chrome-extension://{extension_id}/");
        if !allowed.contains(&origin){allowed.push(origin);}
        let info=serde_json::json!({"name":"com.digitaldiscipline.bridge","description":"Digital Discipline native bridge",
            "path":exe.to_string_lossy(),"type":"stdio","allowed_origins":allowed});
        fs::write(&manifest,serde_json::to_vec_pretty(&info).map_err(|e|e.to_string())?).map_err(|e|e.to_string())?;
        let vendor=match browser.as_str(){"Brave"=>"BraveSoftware\\Brave-Browser","Chrome"=>"Google\\Chrome",_=>"Microsoft\\Edge"};
        let key=format!("HKCU\\Software\\{vendor}\\NativeMessagingHosts\\com.digitaldiscipline.bridge");
        let manifest_text=manifest.to_string_lossy().into_owned();
        let status=Command::new("reg.exe").args(["add",key.as_str(),"/ve","/t","REG_SZ","/d",manifest_text.as_str(),"/f"])
            .status().map_err(|_|"Could not register native host".to_string())?;
        if !status.success(){return Err("Windows rejected per-user native host registration".into());}
        Ok(format!("Paired {browser} with {extension_id}. Restart the browser extension to connect."))
    }
}


/// The extension files ship INSIDE the Windows installer as ordinary Tauri resources.
/// Chromium prohibits silent/unconsented installation of unpacked extensions.
#[tauri::command]
async fn open_extension_folder(app: tauri::AppHandle) -> Result<String,String> {
    #[cfg(not(windows))] { let _=app; return Err("Windows only".into()); }
    #[cfg(windows)] {
        let result=agent_request("POST".into(),"/companion".into(),Some(serde_json::json!({}))).await?;
        let path=PathBuf::from(result["folder"].as_str().ok_or("Companion folder unavailable")?);
        if !path.join("manifest.json").is_file(){return Err("Bundled browser extension not found. Install the updated desktop build.".into());}
        Command::new("explorer.exe").arg(&path).spawn().map_err(|e|format!("Could not open File Explorer: {e}"))?;
        Ok(path.to_string_lossy().into_owned())
    }
}

#[tauri::command]
fn browser_pair_diagnostics(app:tauri::AppHandle,browser:String,extension_id:String)->Result<Value,String>{
    #[cfg(not(windows))] {let _=(app,browser,extension_id);return Err("Windows only".into());}
    #[cfg(windows)] {
        if !["Brave","Chrome","Edge"].contains(&browser.as_str()) {return Err("Unsupported browser".into());}
        let base=std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA unavailable")?;
        let manifest=PathBuf::from(base).join("DigitalDiscipline").join(format!("com.digitaldiscipline.bridge.{browser}.json"));
        let original=app.path().resource_dir().map_err(|e|e.to_string())?.join("bin").join("digital-discipline-agent.exe");
        let info=fs::read(&manifest).ok().and_then(|s|serde_json::from_slice::<Value>(&s).ok());
        let manifest_valid=info.as_ref().and_then(|v|v.get("name")).and_then(Value::as_str)==Some("com.digitaldiscipline.bridge")
          && info.as_ref().and_then(|v|v.get("type")).and_then(Value::as_str)==Some("stdio");
        let actual_path=info.as_ref().and_then(|v|v.get("path")).and_then(Value::as_str).unwrap_or("");
        let host_exists=original.is_file() && PathBuf::from(actual_path).is_file() && PathBuf::from(actual_path)==original;
        let origin=format!("chrome-extension://{extension_id}/");
        let origin_allowed=info.as_ref().and_then(|v|v.get("allowed_origins")).and_then(Value::as_array)
          .is_some_and(|items|items.iter().any(|item|item.as_str()==Some(origin.as_str())));
        let vendor=match browser.as_str(){"Brave"=>"BraveSoftware\\Brave-Browser","Chrome"=>"Google\\Chrome",_=>"Microsoft\\Edge"};
        let key=format!("HKCU\\Software\\{vendor}\\NativeMessagingHosts\\com.digitaldiscipline.bridge");
        let registry=Command::new("reg.exe").args(["query",&key,"/ve"]).output().map_err(|e|e.to_string())?;
        let output=String::from_utf8_lossy(&registry.stdout).to_ascii_lowercase();
        let registry_matches=registry.status.success() && output.contains(&manifest.to_string_lossy().to_ascii_lowercase());
        let api_reachable=TcpStream::connect_timeout(&SocketAddr::from(([127,0,0,1],48761)),Duration::from_millis(450)).is_ok();
        Ok(serde_json::json!({"manifest_exists":manifest.is_file(),"manifest_valid":manifest_valid,
         "registry_matches":registry_matches,"origin_allowed":origin_allowed,"host_exists":host_exists,
         "agent_reachable":api_reachable,"extension_bundled":app.path().resource_dir().ok().is_some_and(|d|d.join("browser-extension").join("manifest.json").is_file()),
         "manifest_path":manifest.to_string_lossy(),"host_path":actual_path,
         "note":"These checks verify local registration, not the live Chromium handshake. The extension popup shows the actual handshake error."}))
    }
}

#[cfg(windows)]
fn send_agent_command(path:&str,body:&str){
    if let Ok(token)=fs::read_to_string(token_path().unwrap_or_default()) {
        if let Ok(mut stream)=TcpStream::connect_timeout(&SocketAddr::from(([127,0,0,1],48761)),Duration::from_millis(500)){
            let _=stream.set_write_timeout(Some(Duration::from_millis(500)));
            let request=format!("POST {path} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",token.trim(),body.len());
            let _=stream.set_read_timeout(Some(Duration::from_secs(2)));
            if stream.write_all(request.as_bytes()).is_ok(){let mut receipt=Vec::new();let _=stream.take(4096).read_to_end(&mut receipt);}
        }
    }
}

// Explicit per-user opt-in: no installer silently installs a startup entry.
#[tauri::command]
fn autostart_status()->Result<bool,String>{
    #[cfg(not(windows))]{return Ok(false);}
    #[cfg(windows)]{
        let result=Command::new("reg.exe").args(["query","HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run","/v","DigitalDisciplineAgent"])
            .status().map_err(|e|e.to_string())?;
        Ok(result.success())
    }
}
#[tauri::command]
fn set_autostart(app:tauri::AppHandle,enabled:bool)->Result<String,String>{
    #[cfg(not(windows))]{let _=(app,enabled);return Err("Windows only".into());}
    #[cfg(windows)]{
        let path=app.path().resource_dir().map_err(|e|e.to_string())?.join("bin").join("digital-discipline-agent.exe");
        if !path.is_file(){return Err("Packaged agent missing".into());}
        let key="HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        let mut command=Command::new("reg.exe");
        if enabled {
            let quoted=format!("\"{}\"",path.display());
            command.args(["add",key,"/v","DigitalDisciplineAgent","/t","REG_SZ","/d",&quoted,"/f"]);
        } else {
            command.args(["delete",key,"/v","DigitalDisciplineAgent","/f"]);
        }
        let result=command.status().map_err(|e|e.to_string())?;
        if !result.success(){return Err("Could not change per-user startup registration".into());}
        Ok(if enabled{"Agent enabled at Windows sign-in"}else{"Agent disabled at Windows sign-in"}.into())
    }
}

#[tauri::command]
async fn export_activity()->Result<Value,String>{
 #[cfg(not(windows))] {return Err("Windows desktop export unavailable".into());}
 #[cfg(windows)] {
  let token=fs::read_to_string(token_path()?).map_err(|_|"Agent token unavailable")?;
  let client=reqwest::Client::builder().timeout(Duration::from_secs(180)).redirect(reqwest::redirect::Policy::none()).build().map_err(|_|"Local export client unavailable")?;
  let response=client.post("http://127.0.0.1:48761/export/snapshot").bearer_auth(token.trim()).json(&serde_json::json!({})).send().await.map_err(|_|"Export could not reach the agent")?;
  if !response.status().is_success(){return Err(format!("Export failed: {}",response.status()));}
  let mut result=response.json::<Value>().await.map_err(|_|"Invalid export result")?;
  let id=result["snapshot_id"].as_str().filter(|id|id.len()==36&&id.bytes().all(|c|c.is_ascii_hexdigit()||c==b'-')).ok_or("Invalid snapshot identifier")?;
  let source=token_path()?.parent().ok_or("Agent directory unavailable")?.join("exports").join(format!("{id}.jsonl"));
  let downloads=PathBuf::from(std::env::var_os("USERPROFILE").ok_or("Windows profile unavailable")?).join("Downloads");
  if !downloads.is_dir(){return Err("Downloads folder unavailable; snapshot remains in DigitalDiscipline/exports".into());}
  let target=downloads.join(format!("Digital-Discipline-Export-{id}.jsonl"));
  fs::copy(&source,&target).map_err(|_|"Could not save export to Downloads")?;
  result["saved_path"]=serde_json::json!(target.to_string_lossy());
  let _=fs::remove_file(source);
  let _=Command::new("explorer.exe").arg("/select,").arg(&target).spawn();
  Ok(result)
 }
}

#[tauri::command]
fn open_data_folder(area:String)->Result<(),String>{
    if !matches!(area.as_str(),"backups"|"updates"){return Err("Unsupported data folder".into());}
    let path=token_path()?.parent().ok_or("Data directory unavailable")?.join(area);
    fs::create_dir_all(&path).map_err(|_|"Data folder unavailable")?;
    #[cfg(windows)] {
        Command::new("explorer.exe").arg(path).spawn().map_err(|_|"Could not open folder")?;
        Ok(())
    }
    #[cfg(not(windows))] {let _=path;Err("Windows folder unavailable".into())}
}

#[tauri::command]
fn set_privacy_mask(app:tauri::AppHandle,enabled:bool){
 #[cfg(windows)] {app.state::<lifecycle::Supervisor>().privacy_mask.store(enabled,std::sync::atomic::Ordering::SeqCst);}
 #[cfg(not(windows))] let _=(app,enabled);
 #[cfg(windows)] if enabled{let _=set_service_hint_window(app,false,false);}
}

#[tauri::command]
fn lifecycle_status(app:tauri::AppHandle)->Value{
 #[cfg(windows)]
 let mut status=app.state::<lifecycle::Supervisor>().status();
 #[cfg(not(windows))]
 let mut status={let _=app;serde_json::json!({"state":"unsupported"})};
 status["resources"]=process_resources::sample("desktop_host_process_only");
 status
}

fn main() {
    #[cfg(windows)] let _single=match lifecycle::SingleInstance::acquire(){Ok(Some(owner))=>owner,Ok(None)=>return,Err(e)=>{eprintln!("{e}");return;}};
    #[cfg(windows)] let activation_event=_single.event();
    tauri::Builder::default()
      .manage(ServiceHintState::default())
      .manage(PresentationState::default())
      .setup(move |app| {
          #[cfg(windows)] {
              use tauri::Emitter;
              use tauri_plugin_global_shortcut::{GlobalShortcutExt,ShortcutState};
              // A conflicting system hotkey must never prevent the app starting.
              app.handle().plugin(tauri_plugin_global_shortcut::Builder::new()
                  .with_handler(|app,_shortcut,event|{
                      if event.state()==ShortcutState::Pressed {
                          if let Some(window)=app.get_webview_window("main") {let _=window.show();}
                          let _=set_presentation(app.clone(),"full".into());
                          if let Some(window)=app.get_webview_window("main") {let _=window.set_focus();}
                          let _=app.emit("dd:global-command",());
                      }
                  }).build())?;
              let _=app.global_shortcut().register("Ctrl+Shift+D");
              let supervisor=lifecycle::Supervisor::new();
              lifecycle::start(app.handle(),&supervisor,activation_event);
              app.manage(supervisor);
              // Persistent Windows tray. Closing the overlay hides it; Quit exits the app
              // and explicitly requests a graceful agent shutdown.
              use tauri::{menu::{Menu,MenuItem},tray::{TrayIconBuilder,TrayIconEvent,MouseButton,MouseButtonState}};
              let open=MenuItem::with_id(app,"open","Open dashboard",true,None::<&str>)?;
              let pause=MenuItem::with_id(app,"pause","Pause tracking",true,None::<&str>)?;
              let resume=MenuItem::with_id(app,"resume","Resume tracking",true,None::<&str>)?;
              let quit=MenuItem::with_id(app,"quit","Quit Digital Discipline",true,None::<&str>)?;
              let modes=MenuItem::with_id(app,"modes","Checking tracking modes…",false,None::<&str>)?;
              let privacy=MenuItem::with_id(app,"privacy","Toggle Presentation Mode",true,None::<&str>)?;
              let supervisor=app.state::<lifecycle::Supervisor>().inner().clone();let modes_copy=modes.clone();
              std::thread::spawn(move||while supervisor.wanted(){
                let status=supervisor.status();let private=status["private_session"]["active"]==true;
                let collection=if status["state"]!="healthy"{"Agent unavailable"}else if private{"Private session"}else if status["tracking_paused"]==true{"Tracking paused"}else{"Tracking on"};
                let masked=supervisor.privacy_mask.load(std::sync::atomic::Ordering::SeqCst);
                let _=modes_copy.set_text(format!("{} · Presentation {}",collection,if masked{"on"}else{"off"}));
                std::thread::sleep(Duration::from_secs(2));
              });
              let menu=Menu::with_items(app,&[&modes,&open,&privacy,&pause,&resume,&quit])?;
              TrayIconBuilder::new()
                .icon(app.default_window_icon().ok_or_else(||std::io::Error::other("Window icon missing"))?.clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app,event|match event.id().as_ref(){
                   "open"=>{if let Some(w)=app.get_webview_window("main"){let _=w.show();}let _=set_presentation(app.clone(),"full".into());if let Some(w)=app.get_webview_window("main"){let _=w.set_focus();}let _=app.emit("dd:open-dashboard",());},
                   "privacy"=>{let _=app.emit("dd:toggle-privacy",());},
                   "pause"=>send_agent_command("/tracking/pause",r#"{"paused":true}"#),
                   "resume"=>send_agent_command("/tracking/pause",r#"{"paused":false}"#),
                   "quit"=>{app.state::<lifecycle::Supervisor>().stop();send_agent_command("/shutdown","{}");app.exit(0)},
                   _=>{},
                })
                .on_tray_icon_event(|tray,event|{if let TrayIconEvent::Click{button:MouseButton::Left,button_state:MouseButtonState::Up,..}=event{
                    let app=tray.app_handle();if let Some(w)=app.get_webview_window("main"){let _=w.show();}let _=set_presentation(app.clone(),"full".into());if let Some(w)=app.get_webview_window("main"){let _=w.set_focus();}let _=app.emit("dd:open-dashboard",());
                }})
                .build(app)?;

          }
          let _ = set_presentation(app.handle().clone(), "pill".to_owned());
          Ok(())
      })
      .on_window_event(|window,event| {
          if window.label()=="service-hint"&&matches!(event,tauri::WindowEvent::Destroyed){if let Some(state)=window.app_handle().try_state::<ServiceHintState>(){if let Ok(mut payload)=state.payload.lock(){*payload=None;};}}
          if window.label()=="main"&&matches!(event,tauri::WindowEvent::Moved(_)|tauri::WindowEvent::ScaleFactorChanged{..}){
              let app=window.app_handle();let current=window.current_monitor().ok().flatten().map(|m|monitor_key(&m));
              let requested=app.state::<PresentationState>().current.lock().ok().and_then(|state|(!state.0.is_empty()&&current.is_some()&&state.1!=current).then(||state.0.clone()));
              if let Some(mode)=requested{let _=set_presentation(app.clone(),mode);}
          }
          if window.label()=="service-hint"&&matches!(event,tauri::WindowEvent::Focused(false)){let _=window.app_handle().emit_to("service-hint","dd:notice-away",());}
          #[cfg(windows)] if let tauri::WindowEvent::CloseRequested {api,..}=event {if window.label()=="main"{api.prevent_close();let _=window.app_handle().emit_to("main","dd:collapse-request",());}}
          #[cfg(not(windows))] let _=(window,event);
      })
      .invoke_handler(tauri::generate_handler![agent_request,open_data_folder,set_presentation,drag_window,hide_to_tray,open_extension_folder,autostart_status,set_autostart,lifecycle_status,export_activity,set_privacy_mask,deliver_service_hint,deliver_desktop_notice,read_service_hint,set_service_hint_window,open_service_review,open_notice_page])
      .run(tauri::generate_context!())
      .expect("Digital Discipline failed to start");
}

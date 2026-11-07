#![cfg_attr(all(windows,not(debug_assertions)), windows_subsystem = "windows")]
mod ipc_routes;
#[cfg(windows)] mod lifecycle;
use serde_json::Value;
use std::{fs, path::PathBuf, time::Duration};
#[cfg(windows)] use std::io::{Read,Write};
#[cfg(windows)] use std::{net::{TcpStream,SocketAddr},process::Command};
use tauri::{Manager, LogicalSize, Size};

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
    let client=reqwest::Client::builder().timeout(Duration::from_secs(if path=="/coach" {58}else{6})).build().map_err(|e|e.to_string())?;
    let mut request=client.request(method.parse().map_err(|_|"Invalid HTTP method")?,format!("http://127.0.0.1:48761{}",path)).bearer_auth(token.trim());
    if method=="POST" {request=request.json(&body.unwrap_or(Value::Null));}
    let response=request.send().await.map_err(|_|"Local agent unavailable".to_owned())?;
    if !response.status().is_success() {return Err(format!("Agent reported {}",response.status()));}
    response.json::<Value>().await.map_err(|_|"Invalid agent response".into())
}

#[tauri::command]
fn set_presentation(app:tauri::AppHandle,mode:String)->Result<(),String> {
    let w=app.get_webview_window("main").ok_or("Window unavailable")?;
    let (mut width, mut height): (f64, f64) = match mode.as_str() {
        "pill" => (280.0,56.0), "quick" => (410.0,430.0),
        "full"|"popout" => (1220.0,790.0), _ => return Err("Unsupported mode".into()),
    };
    if mode=="full" || mode=="popout" {
        if let Some(monitor)=w.current_monitor().map_err(|e|e.to_string())? {
            let scale=w.scale_factor().map_err(|e|e.to_string())?;
            width=width.min(monitor.size().width as f64 / scale * 0.90);
            height=height.min(monitor.size().height as f64 / scale * 0.86);
        }
    }
    let popout=mode=="popout";
    // A real maximize operation is available only for the decorated pop-out window.
    if !popout {let _=w.unmaximize();}
    w.set_size(Size::Logical(LogicalSize::new(width,height))).map_err(|e|e.to_string())?;
    w.set_always_on_top(!popout).map_err(|e|e.to_string())?;
    w.set_skip_taskbar(!popout).map_err(|e|e.to_string())?;
    w.set_decorations(popout).map_err(|e|e.to_string())?;
    if popout {w.maximize().map_err(|e|e.to_string())?;}
    // Keep the user's chosen position when switching between pill, panel, and workspace.
    // Clamp after resizing so a larger panel cannot disappear beyond a monitor edge.
    if let Some(monitor)=w.current_monitor().map_err(|e|e.to_string())? {
        let current=w.outer_position().map_err(|e|e.to_string())?;
        let size=w.outer_size().map_err(|e|e.to_string())?;
        let bounds=monitor.size();
        let left=monitor.position().x+8;
        let top=monitor.position().y+8;
        let right=(monitor.position().x+bounds.width as i32-size.width as i32-8).max(left);
        let bottom=(monitor.position().y+bounds.height as i32-size.height as i32-8).max(top);
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
fn open_extension_folder(app: tauri::AppHandle) -> Result<String,String> {
    #[cfg(not(windows))] { let _=app; return Err("Windows only".into()); }
    #[cfg(windows)] {
        let path=app.path().resource_dir().map_err(|e|e.to_string())?.join("browser-extension");
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
fn set_privacy_mask(app:tauri::AppHandle,enabled:bool){
 #[cfg(windows)] {app.state::<lifecycle::Supervisor>().privacy_mask.store(enabled,std::sync::atomic::Ordering::SeqCst);}
 #[cfg(not(windows))] let _=(app,enabled);
}

#[tauri::command]
fn lifecycle_status(app:tauri::AppHandle)->Value{
 #[cfg(windows)] {return app.state::<lifecycle::Supervisor>().status();}
 #[cfg(not(windows))] {let _=app;serde_json::json!({"state":"unsupported","note":"Windows desktop supervisor"})}
}

fn main() {
    #[cfg(windows)] let _single=match lifecycle::SingleInstance::acquire(){Ok(Some(owner))=>owner,Ok(None)=>return,Err(e)=>{eprintln!("{e}");return;}};
    #[cfg(windows)] let activation_event=_single.event();
    tauri::Builder::default()
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
          #[cfg(windows)] if let tauri::WindowEvent::CloseRequested {api,..}=event {api.prevent_close();let _=window.hide();}
          #[cfg(not(windows))] let _=(window,event);
      })
      .invoke_handler(tauri::generate_handler![agent_request,set_presentation,drag_window,hide_to_tray,open_extension_folder,autostart_status,set_autostart,lifecycle_status,export_activity,set_privacy_mask])
      .run(tauri::generate_context!())
      .expect("Digital Discipline failed to start");
}

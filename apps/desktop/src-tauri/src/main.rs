#![cfg_attr(all(windows,not(debug_assertions)), windows_subsystem = "windows")]
use serde_json::Value;
use std::{fs, path::PathBuf, time::Duration};
#[cfg(windows)] use std::io::Write;
#[cfg(windows)] use std::{net::{TcpStream,SocketAddr},process::Command};
use tauri::{Manager, LogicalSize, Size};

fn token_path() -> Result<PathBuf,String> {
    let dir=std::env::var_os("LOCALAPPDATA").ok_or("LOCALAPPDATA unavailable")?;
    Ok(PathBuf::from(dir).join("DigitalDiscipline").join("agent.token"))
}

#[tauri::command]
async fn agent_request(method:String,path:String,body:Option<Value>) -> Result<Value,String> {
    // Strict IPC allowlist prevents a compromised WebView from becoming an arbitrary HTTP proxy.
    let allowed=match (method.as_str(),path.as_str()) {
        ("GET","/state"|"/export"|"/videos"|"/insights"|"/categories"|"/goals"|"/workflows"|"/settings"|"/classifications"|"/devices"|"/devices/export"|"/rules/audit"|"/browser-pairs"|"/coach/credentials") => true,
        ("GET",p) if p.starts_with("/sessions/") && p.len()==20 && p[10..].bytes().all(|c|c.is_ascii_digit()||c==b'-') => true,
        ("POST","/tracking/pause"|"/rules"|"/categories/override"|"/goals"|"/workflows/profiles"|"/workflows/tasks"|"/focus/start"|"/focus/end"|"/focus/hint/choice"|"/settings"|"/coach"|"/coach/credentials"|"/classifications/config"|"/devices/import"|"/rules/preview"|"/browser-pairs") => true,
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
    };
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

#[cfg(windows)]
fn start_bundled_agent(app:&tauri::AppHandle) {
    // UI can be closed or crash without terminating the headless native agent.
    use std::os::windows::process::CommandExt;
    let exe=app.path().resource_dir().ok().map(|d|d.join("bin").join("digital-discipline-agent.exe"));
    if let Some(exe)=exe.filter(|e|e.is_file()) {
        let addr=SocketAddr::from(([127,0,0,1],48761));
        if TcpStream::connect_timeout(&addr,Duration::from_millis(350)).is_err() {
            // CREATE_NO_WINDOW. Never request elevated permissions here.
            let _=Command::new(exe).creation_flags(0x08000000).spawn();
        }
    }
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
            let _=stream.write_all(request.as_bytes());
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

fn main() {
    tauri::Builder::default()
      .setup(|app| {
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
              start_bundled_agent(app.handle());
              // Persistent Windows tray. Closing the overlay hides it; Quit exits the app
              // and explicitly requests a graceful agent shutdown.
              use tauri::{menu::{Menu,MenuItem},tray::{TrayIconBuilder,TrayIconEvent,MouseButton,MouseButtonState}};
              let open=MenuItem::with_id(app,"open","Open dashboard",true,None::<&str>)?;
              let pause=MenuItem::with_id(app,"pause","Pause tracking",true,None::<&str>)?;
              let resume=MenuItem::with_id(app,"resume","Resume tracking",true,None::<&str>)?;
              let quit=MenuItem::with_id(app,"quit","Quit Digital Discipline",true,None::<&str>)?;
              let menu=Menu::with_items(app,&[&open,&pause,&resume,&quit])?;
              TrayIconBuilder::new()
                .icon(app.default_window_icon().ok_or_else(||std::io::Error::other("Window icon missing"))?.clone())
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(|app,event|match event.id().as_ref(){
                   "open"=>{if let Some(w)=app.get_webview_window("main"){let _=w.show();}let _=set_presentation(app.clone(),"full".into());if let Some(w)=app.get_webview_window("main"){let _=w.set_focus();}let _=app.emit("dd:open-dashboard",());},
                   "pause"=>send_agent_command("/tracking/pause",r#"{"paused":true}"#),
                   "resume"=>send_agent_command("/tracking/pause",r#"{"paused":false}"#),
                   "quit"=>{send_agent_command("/shutdown","{}");app.exit(0)},
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
      .invoke_handler(tauri::generate_handler![agent_request,set_presentation,drag_window,hide_to_tray,open_extension_folder,autostart_status,set_autostart])
      .run(tauri::generate_context!())
      .expect("Digital Discipline failed to start");
}

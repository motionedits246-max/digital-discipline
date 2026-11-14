use std::{
    env, fs, io,
    os::linux::net::SocketAddrExt,
    os::unix::net::{SocketAddr, UnixListener, UnixStream},
    path::PathBuf,
    process::Command,
};
use tauri::Manager;

pub fn bind_desktop() -> io::Result<Option<UnixListener>> {
    use std::hash::{DefaultHasher, Hash, Hasher};
    let mut key = DefaultHasher::new();
    crate::local_data::directory()?.hash(&mut key);
    let address = SocketAddr::from_abstract_name(format!("digital-discipline-desktop-{:016x}", key.finish()))?;
    match UnixListener::bind_addr(&address) {
        Ok(listener) => {
            listener.set_nonblocking(true)?;
            Ok(Some(listener))
        }
        Err(error) if error.kind() == io::ErrorKind::AddrInUse => {
            UnixStream::connect_addr(&address)?;
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

pub fn agent_binary(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let installed = app.path().resource_dir().map_err(|e| e.to_string())?.join("bin/digital-discipline-agent");
    if installed.is_file() {
        return Ok(installed);
    }
    #[cfg(debug_assertions)]
    if let Ok(mut path) = env::current_exe() {
        path.set_file_name("digital-discipline-agent");
        if path.is_file() {
            return Ok(path);
        }
    }
    Err("Packaged agent missing".into())
}

fn autostart_path() -> io::Result<PathBuf> {
    let base = env::var_os("XDG_CONFIG_HOME").map(PathBuf::from).filter(|p| p.is_absolute())
        .map(Ok).unwrap_or_else(|| crate::local_data::home().map(|p| p.join(".config")))?;
    Ok(base.join("autostart/dev.digitaldiscipline.agent.desktop"))
}

pub fn autostart_status() -> Result<bool, String> {
    autostart_path().map(|p| p.is_file()).map_err(|e| e.to_string())
}

pub fn set_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<String, String> {
    let path = autostart_path().map_err(|e| e.to_string())?;
    if enabled {
        let agent = agent_binary(app)?;
        let agent = agent.to_str().ok_or("Agent path must be UTF-8")?;
        if agent.contains(['\n', '\r']) {
            return Err("Unsupported agent path".into());
        }
        let escaped = agent.replace('\\', "\\\\\\\\").replace('"', "\\\\\"").replace('$', "\\\\$").replace('`', "\\\\`").replace('%', "%%");
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let staging = path.with_extension("tmp");
        fs::write(&staging, format!("[Desktop Entry]\nType=Application\nName=Digital Discipline agent\nExec=\"{escaped}\"\nTerminal=false\n"))
            .and_then(|()| fs::rename(staging, path)).map_err(|e| e.to_string())?;
    } else if let Err(error) = fs::remove_file(path) {
        if error.kind() != io::ErrorKind::NotFound {
            return Err(error.to_string());
        }
    }
    Ok(if enabled { "Agent enabled at desktop sign-in" } else { "Agent disabled at desktop sign-in" }.into())
}

pub fn downloads() -> io::Result<PathBuf> {
    if let Ok(result) = Command::new("xdg-user-dir").arg("DOWNLOAD").output() {
        if result.status.success() {
            let path = PathBuf::from(String::from_utf8_lossy(&result.stdout).trim());
            if path.is_absolute() {
                return Ok(path);
            }
        }
    }
    Ok(crate::local_data::home()?.join("Downloads"))
}

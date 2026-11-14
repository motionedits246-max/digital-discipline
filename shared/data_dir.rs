use std::{env, io, path::PathBuf};

pub fn directory() -> io::Result<PathBuf> {
    #[cfg(target_os = "linux")]
    {
        if let Some(path) = env::var_os("XDG_DATA_HOME").map(PathBuf::from).filter(|p| p.is_absolute()) {
            return Ok(path.join("digital-discipline"));
        }
        return home().map(|path| path.join(".local/share/digital-discipline"));
    }
    #[cfg(not(target_os = "linux"))]
    {
        let base = env::var_os("LOCALAPPDATA").or_else(|| env::var_os("HOME"))
            .ok_or_else(|| io::Error::other("Local application data folder unavailable"))?;
        Ok(PathBuf::from(base).join("DigitalDiscipline"))
    }
}

#[cfg(target_os = "linux")]
pub fn home() -> io::Result<PathBuf> {
    env::var_os("HOME").map(PathBuf::from).filter(|p| p.is_absolute())
        .ok_or_else(|| io::Error::other("Home directory unavailable"))
}

#[cfg(target_os = "linux")]
pub fn x11_session() -> bool {
    env::var_os("DISPLAY").is_some_and(|display| !display.is_empty())
        && env::var_os("XDG_SESSION_TYPE").is_none_or(|kind| kind == "x11")
        && env::var_os("WAYLAND_DISPLAY").is_none_or(|display| display.is_empty())
}

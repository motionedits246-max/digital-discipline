use crate::domain::Observation;
use std::{env, fs, io::Read, path::PathBuf, process::Command};

fn output(program: &str, args: &[&str]) -> Option<String> {
    let result = Command::new("timeout").args(["1s", program]).args(args).output().ok()?;
    result.status.success().then(|| String::from_utf8_lossy(&result.stdout).into_owned())
}

fn foreground() -> Option<(String, u32, String)> {
    let info = output("xprop", &["-root", "_NET_ACTIVE_WINDOW"])?;
    let id = info.split_whitespace().last()?;
    if u32::from_str_radix(id.strip_prefix("0x")?, 16).ok()? == 0 {
        return None;
    }
    let info = output("xprop", &["-id", id, "_NET_WM_PID"])?;
    let pid = info.split_once('=')?.1.trim().parse::<u32>().ok()?;
    let executable = fs::read_link(format!("/proc/{pid}/exe")).ok()?;
    let app = executable.file_name()?.to_str()?.to_ascii_lowercase();
    if app.is_empty() || app.len() > 100 || !app.bytes().all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
        || ["gnome-shell", "plasmashell", "kscreenlocker_greet", "xfce4-screensaver",
            "light-locker", "mate-screensaver", "cinnamon-screensaver", "i3lock", "xsecurelock"].contains(&app.as_str()) {
        return None;
    }
    Some((id.to_owned(), pid, app))
}

pub fn observe(at_ms: i64, idle_limit_ms: u32) -> Option<Observation> {
    if !crate::local_data::x11_session() {
        return None;
    }
    let (_, _, app) = foreground()?;
    let idle_ms = output("xprintidle", &[])?.trim().parse::<u64>().ok()?;
    Some(Observation { app, idle: idle_ms > u64::from(idle_limit_ms), domain: None, profile_id: None, at_ms })
}

pub fn minimize_foreground(expected: &str) -> bool {
    if !crate::local_data::x11_session() || !crate::inbox::safe_native_app(expected) {
        return false;
    }
    let Some((id, pid, app)) = foreground() else { return false; };
    if app != expected || pid == std::process::id() || foreground() != Some((id.clone(), pid, app)) {
        return false;
    }
    output("xdotool", &["windowminimize", &id]).is_some()
}


/// Look up the foreground executable in desktop launcher metadata. Never read window titles.
pub fn application_description(expected: &str) -> String {
    if !crate::local_data::x11_session() || foreground().is_none_or(|(_, _, app)| app != expected) {
        return String::new();
    }
    let system_dirs = env::var_os("XDG_DATA_DIRS").unwrap_or_else(|| "/usr/local/share:/usr/share".into());
    let user_dir = env::var_os("XDG_DATA_HOME").map(PathBuf::from).filter(|path| path.is_absolute())
        .or_else(|| crate::local_data::home().ok().map(|home| home.join(".local/share")));
    for directory in user_dir.into_iter().chain(env::split_paths(&system_dirs)).take(12) {
        if !directory.is_absolute() {
            continue;
        }
        let Ok(entries) = fs::read_dir(directory.join("applications")) else { continue; };
        for entry in entries.take(1200).flatten() {
            if entry.path().extension().is_none_or(|extension| extension != "desktop") {
                continue;
            }
            let Ok(metadata) = entry.metadata() else { continue; };
            if !metadata.is_file() || metadata.len() > 64 * 1024 {
                continue;
            }
            let mut text = String::new();
            if fs::File::open(entry.path()).and_then(|file| file.take(64 * 1024).read_to_string(&mut text)).is_err() {
                continue;
            }
            if let Some(description) = desktop_description(&text, expected) {
                return description;
            }
        }
    }
    String::new()
}

fn desktop_description(text: &str, expected: &str) -> Option<String> {
    let mut fields = text.lines().map(str::trim).skip_while(|line| *line != "[Desktop Entry]");
    fields.next()?;
    let (mut executable, mut name, mut generic, mut comment) = ("", "", "", "");
    for line in fields.take_while(|line| !line.starts_with('[')) {
        match line.split_once('=') {
            Some(("Type", body)) if body != "Application" => return None,
            Some(("Hidden", "true")) => return None,
            Some(("Exec", body)) => executable = body,
            Some(("Name", body)) => name = body,
            Some(("GenericName", body)) => generic = body,
            Some(("Comment", body)) => comment = body,
            _ => {}
        }
    }
    let executable = if let Some(quoted) = executable.strip_prefix('"') {
        quoted.split_once('"')?.0
    } else {
        executable.split_whitespace().next()?
    };
    if !executable.rsplit('/').next()?.eq_ignore_ascii_case(expected) {
        return None;
    }
    let description = crate::service_discovery::clean_description(&format!("{name} {generic} {comment}"));
    (!description.is_empty()).then_some(description)
}

#[cfg(test)]
mod tests {
    use super::desktop_description;

    #[test]
    fn desktop_metadata_requires_the_observed_executable_and_ignores_actions() {
        let launcher = r#"[Desktop Entry]
Type=Application
Exec="/opt/Editor Suite/code-app" %F
Name=Editor Suite
GenericName=Code editor
[Desktop Action Inspect]
Comment=Ignore action metadata
"#;
        assert_eq!(desktop_description(launcher, "code-app").as_deref(), Some("Editor Suite Code editor"));
        assert_eq!(desktop_description(launcher, "other-app"), None);
        assert_eq!(desktop_description("[Desktop Entry]
Hidden=true
Exec=code-app
Name=Wrong", "code-app"), None);
    }
}

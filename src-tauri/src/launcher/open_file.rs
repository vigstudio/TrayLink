use std::path::{Component, Path, PathBuf};
use std::process::Command;

use super::LauncherError;

const BLOCKED_PREFIXES: &[&str] = &[
    "/etc",
    "/sys",
    "/proc",
    "/bin",
    "/sbin",
    "/usr/bin",
    "/usr/sbin",
    "C:\\Windows\\System32",
    "C:\\Windows\\SysWOW64",
];

pub fn open_file(path: &str) -> Result<(), LauncherError> {
    let normalized = normalize_path(path)?;

    if is_blocked(&normalized) {
        return Err(LauncherError::PathNotAllowed(path.to_string()));
    }

    if !Path::new(&normalized).exists() {
        return Err(LauncherError::PathNotFound(normalized));
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", "start", "", &normalized])
            .spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    }

    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .arg(&normalized)
            .spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    }

    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open")
            .arg(&normalized)
            .spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    }

    Ok(())
}

fn normalize_path(path: &str) -> Result<String, LauncherError> {
    let path_buf = PathBuf::from(path);
    let mut normalized = PathBuf::new();

    for component in path_buf.components() {
        match component {
            Component::ParentDir => {
                return Err(LauncherError::PathNotAllowed(path.to_string()));
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }

    Ok(normalized.to_string_lossy().to_string())
}

fn is_blocked(path: &str) -> bool {
    let lower = path.to_lowercase();
    BLOCKED_PREFIXES
        .iter()
        .any(|prefix| lower.starts_with(&prefix.to_lowercase()))
}

use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::config::AppEntry;
use super::browser::{is_browser, validate_url};
use super::LauncherError;

pub fn open_app(
    app_key: &str,
    apps: &HashMap<String, AppEntry>,
    url_override: Option<&str>,
) -> Result<(), LauncherError> {
    let entry = apps
        .get(app_key)
        .ok_or_else(|| LauncherError::AppNotAllowed(app_key.to_string()))?;

    let url = url_override
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .or_else(|| entry.url.as_deref().map(str::trim).filter(|value| !value.is_empty()));

    if let Some(url) = url {
        validate_url(url).map_err(|err| LauncherError::LaunchFailed(err))?;
    }

    launch_path(&entry.path, &entry.args, url, entry.url_enabled)
}

fn launch_path(
    path: &str,
    args: &[String],
    url: Option<&str>,
    url_enabled: bool,
) -> Result<(), LauncherError> {
    let with_url = url.is_some() && (is_browser(path) || url_enabled);

    #[cfg(target_os = "windows")]
    {
        if with_url {
            if path.ends_with(".exe") || path.contains('\\') || path.contains('/') {
                let mut cmd = Command::new(path);
                cmd.arg(url.unwrap());
                cmd.args(args);
                cmd.spawn()
                    .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
            } else {
                Command::new("cmd")
                    .args(["/C", "start", "", path, url.unwrap()])
                    .spawn()
                    .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
            }
            return Ok(());
        }

        if path.ends_with(".exe") || path.contains('\\') || path.contains('/') {
            let mut cmd = Command::new(path);
            cmd.args(args);
            cmd.spawn()
                .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        } else {
            Command::new("cmd")
                .args(["/C", "start", "", path])
                .spawn()
                .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if with_url {
            let app_name = if path.ends_with(".app") {
                Path::new(path)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .unwrap_or(path)
            } else {
                path
            };

            let mut cmd = Command::new("open");
            cmd.args(["-a", app_name, url.unwrap()]);
            cmd.args(args);
            cmd.spawn()
                .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
            return Ok(());
        }

        if path.ends_with(".app") || path.contains('/') {
            let mut cmd = Command::new("open");
            cmd.arg(path);
            cmd.args(args);
            cmd.spawn()
                .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        } else {
            let mut cmd = Command::new("open");
            cmd.args(["-a", path]);
            cmd.args(args);
            cmd.spawn()
                .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        if with_url {
            if path.ends_with(".desktop") || !path.contains('/') {
                let mut cmd = Command::new("xdg-open");
                cmd.arg(url.unwrap());
                cmd.spawn()
                    .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
            } else {
                let mut cmd = Command::new(path);
                cmd.arg(url.unwrap());
                cmd.args(args);
                cmd.spawn()
                    .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
            }
            return Ok(());
        }

        if path.ends_with(".desktop") || !path.contains('/') {
            Command::new("xdg-open")
                .arg(path)
                .spawn()
                .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        } else {
            let mut cmd = Command::new(path);
            cmd.args(args);
            cmd.spawn()
                .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        }
    }

    Ok(())
}

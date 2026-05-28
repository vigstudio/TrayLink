use std::collections::HashMap;

#[cfg(target_os = "macos")]
use std::path::Path;
#[cfg(any(target_os = "macos", target_os = "linux"))]
use std::process::Command;

use crate::config::AppEntry;
use super::browser::{is_browser, validate_url};
use super::browser_profiles;
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
        validate_url(url).map_err(LauncherError::LaunchFailed)?;
    }

    launch_path(
        &entry.path,
        &entry.args,
        url,
        entry.url_enabled,
        entry.browser_profile.as_deref(),
    )
}

fn launch_path(
    path: &str,
    args: &[String],
    url: Option<&str>,
    url_enabled: bool,
    browser_profile: Option<&str>,
) -> Result<(), LauncherError> {
    let with_url = url.is_some() && (is_browser(path) || url_enabled);
    let profile_args = browser_profile
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|profile| browser_profiles::build_profile_args(path, profile))
        .unwrap_or_default();
    let use_browser_launch = is_browser(path) && (!profile_args.is_empty() || with_url);

    #[cfg(target_os = "windows")]
    {
        let launch_path = crate::apps::resolve_launch_path(path);
        if use_browser_launch || !profile_args.is_empty() {
            launch_windows_browser(
                &launch_path,
                args,
                url,
                with_url,
                &profile_args,
            )?;
            return Ok(());
        }

        crate::apps::launch_windows_path(&launch_path, args, url, with_url)
            .map_err(LauncherError::LaunchFailed)?;
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        if use_browser_launch {
            launch_macos_browser(path, args, url, with_url, &profile_args)?;
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
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        if use_browser_launch {
            launch_linux_browser(path, args, url, with_url, &profile_args)?;
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
        return Ok(());
    }
}

#[cfg(target_os = "macos")]
fn launch_macos_browser(
    path: &str,
    args: &[String],
    url: Option<&str>,
    with_url: bool,
    profile_args: &[String],
) -> Result<(), LauncherError> {
    use std::thread;
    use std::time::Duration;

    let app_name = if path.ends_with(".app") {
        Path::new(path)
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or(path)
    } else {
        path
    };

    let launched = launch_macos_browser_process(path, app_name, args, url, with_url, profile_args);

    if launched.is_err() {
        let mut cmd = Command::new("open");
        cmd.arg("-a").arg(app_name).arg("--args");
        cmd.args(profile_args);
        cmd.args(args);
        if with_url {
            cmd.arg(url.unwrap());
        }
        cmd.spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    }

    thread::sleep(Duration::from_millis(450));
    crate::macos::input::activate_and_focus(path)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn launch_macos_browser_process(
    app_path: &str,
    app_name: &str,
    args: &[String],
    url: Option<&str>,
    with_url: bool,
    profile_args: &[String],
) -> Result<(), LauncherError> {
    let executable = macos_app_executable(app_path, app_name)?;
    let mut cmd = Command::new(&executable);
    cmd.args(profile_args);
    cmd.args(args);
    if with_url {
        cmd.arg(url.unwrap());
    }
    cmd.spawn()
        .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn macos_app_executable(app_path: &str, app_name: &str) -> Result<std::path::PathBuf, LauncherError> {
    use std::fs;
    use std::path::Path;

    if !app_path.ends_with(".app") {
        return Err(LauncherError::LaunchFailed(
            "not an app bundle path".to_string(),
        ));
    }

    let macos_dir = Path::new(app_path).join("Contents/MacOS");
    if !macos_dir.is_dir() {
        return Err(LauncherError::LaunchFailed(
            "missing Contents/MacOS".to_string(),
        ));
    }

    let preferred = macos_dir.join(app_name);
    if preferred.is_file() {
        return Ok(preferred);
    }

    let mut candidates = Vec::new();
    for entry in fs::read_dir(&macos_dir).map_err(|e| LauncherError::LaunchFailed(e.to_string()))? {
        let entry = entry.map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        let path = entry.path();
        if path.is_file() {
            candidates.push(path);
        }
    }

    candidates
        .into_iter()
        .find(|path| path.extension().is_none())
        .ok_or_else(|| LauncherError::LaunchFailed("browser executable not found".to_string()))
}

#[cfg(target_os = "linux")]
fn launch_linux_browser(
    path: &str,
    args: &[String],
    url: Option<&str>,
    with_url: bool,
    profile_args: &[String],
) -> Result<(), LauncherError> {
    if path.ends_with(".desktop") || !path.contains('/') {
        let mut cmd = Command::new("xdg-open");
        if with_url {
            cmd.arg(url.unwrap());
        } else {
            cmd.arg(path);
        }
        cmd.spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        return Ok(());
    }

    let mut cmd = Command::new(path);
    cmd.args(profile_args);
    cmd.args(args);
    if with_url {
        cmd.arg(url.unwrap());
    }
    cmd.spawn()
        .map_err(|e| LauncherError::LaunchFailed(e.to_string()))
}

#[cfg(target_os = "windows")]
fn launch_windows_browser(
    path: &std::path::Path,
    args: &[String],
    url: Option<&str>,
    with_url: bool,
    profile_args: &[String],
) -> Result<(), LauncherError> {
    use std::process::Command;

    let path_str = path.to_string_lossy();
    if path.extension().and_then(|ext| ext.to_str()) == Some("exe") && path.is_file() {
        let mut cmd = Command::new(path.as_os_str());
        cmd.args(profile_args);
        cmd.args(args);
        if with_url {
            cmd.arg(url.unwrap());
        }
        cmd.spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        return Ok(());
    }

    let mut launch_args = profile_args.to_vec();
    launch_args.extend_from_slice(args);
    if with_url {
        launch_args.push(url.unwrap().to_string());
    }

    Command::new("cmd")
        .args(["/C", "start", "", "", &*path_str])
        .args(&launch_args)
        .spawn()
        .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    Ok(())
}

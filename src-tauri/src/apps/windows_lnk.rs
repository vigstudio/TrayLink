use std::path::{Path, PathBuf};

#[cfg(target_os = "windows")]
use std::process::Command;

/// Resolve `.lnk` shortcut to its target path (usually `.exe`). Returns input unchanged on other platforms or extensions.
pub fn resolve_launch_path(path: &str) -> PathBuf {
    let candidate = PathBuf::from(path);
    #[cfg(target_os = "windows")]
    {
        if is_lnk(&candidate) {
            if let Some(target) = resolve_lnk_target(&candidate) {
                return target;
            }
        }
    }
    candidate
}

#[cfg(target_os = "windows")]
pub fn is_lnk(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"))
}

#[cfg(not(target_os = "windows"))]
pub fn is_lnk(_path: &Path) -> bool {
    false
}

#[cfg(target_os = "windows")]
pub fn resolve_lnk_target(lnk: &Path) -> Option<PathBuf> {
    if !lnk.is_file() || !is_lnk(lnk) {
        return None;
    }

    let lnk_escaped = lnk.display().to_string().replace('\'', "''");
    let script = format!(
        "$s = (New-Object -ComObject WScript.Shell).CreateShortcut('{lnk_escaped}'); \
         if ($s.TargetPath) {{ Write-Output $s.TargetPath }}"
    );

    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-WindowStyle",
            "Hidden",
            "-Command",
            &script,
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let target = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if target.is_empty() {
        return None;
    }

    let path = PathBuf::from(target);
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[cfg(target_os = "windows")]
pub fn launch_windows_path(path: &Path, args: &[String], url: Option<&str>, with_url: bool) -> Result<(), String> {
    use std::process::Command;

    let path_str = path.to_string_lossy();

    if with_url {
        let url = url.unwrap();
        if is_lnk(path) || path.extension().and_then(|e| e.to_str()) != Some("exe") {
            Command::new("cmd")
                .args(["/C", "start", "", &*path_str, url])
                .spawn()
                .map_err(|e| e.to_string())?;
        } else {
            let mut cmd = Command::new(&*path_str);
            cmd.arg(url);
            cmd.args(args);
            cmd.spawn().map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    if is_lnk(path) {
        Command::new("cmd")
            .args(["/C", "start", "", "", &*path_str])
            .spawn()
            .map_err(|e| e.to_string())?;
        return Ok(());
    }

    if path.extension().and_then(|e| e.to_str()) == Some("exe") && path.is_file() {
        let mut cmd = Command::new(&*path_str);
        cmd.args(args);
        cmd.spawn().map_err(|e| e.to_string())?;
        return Ok(());
    }

    Command::new("cmd")
        .args(["/C", "start", "", "", &*path_str])
        .spawn()
        .map_err(|e| e.to_string())?;
    Ok(())
}

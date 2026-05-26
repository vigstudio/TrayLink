use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct InstalledApp {
    pub name: String,
    pub path: String,
}

pub fn list_installed_apps() -> Vec<InstalledApp> {
    let mut apps = Vec::new();

    #[cfg(target_os = "macos")]
    collect_macos_apps(&mut apps);

    #[cfg(target_os = "linux")]
    collect_linux_apps(&mut apps);

    #[cfg(target_os = "windows")]
    collect_windows_apps(&mut apps);

    apps.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    apps.dedup_by(|a, b| a.path == b.path);
    apps
}

#[cfg(target_os = "macos")]
fn collect_macos_apps(apps: &mut Vec<InstalledApp>) {
    let mut dirs = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
    ];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join("Applications"));
    }

    for dir in dirs {
        collect_app_bundles(&dir, apps);
    }
}

#[cfg(target_os = "macos")]
fn collect_app_bundles(dir: &Path, apps: &mut Vec<InstalledApp>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("app") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Unknown")
            .to_string();

        apps.push(InstalledApp {
            name,
            path: path.to_string_lossy().to_string(),
        });
    }
}

#[cfg(target_os = "linux")]
fn collect_linux_apps(apps: &mut Vec<InstalledApp>) {
    let mut dirs = vec![PathBuf::from("/usr/share/applications")];
    if let Ok(home) = std::env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share/applications"));
    }

    for dir in dirs {
        collect_desktop_files(&dir, apps);
    }
}

#[cfg(target_os = "linux")]
fn collect_desktop_files(dir: &Path, apps: &mut Vec<InstalledApp>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("desktop") {
            continue;
        }
        if let Some(app) = parse_desktop_file(&path) {
            apps.push(app);
        }
    }
}

#[cfg(target_os = "linux")]
fn parse_desktop_file(path: &Path) -> Option<InstalledApp> {
    let content = fs::read_to_string(path).ok()?;
    let mut name = None;
    let mut exec = None;
    let mut no_display = false;

    for line in content.lines() {
        if line.starts_with("Name=") {
            name = Some(line.trim_start_matches("Name=").trim().to_string());
        } else if line.starts_with("Exec=") {
            exec = Some(line.trim_start_matches("Exec=").trim().to_string());
        } else if line.starts_with("NoDisplay=true") {
            no_display = true;
        }
    }

    if no_display {
        return None;
    }

    let name = name?;
    let exec = exec?;
    let exec = exec.split_whitespace().next()?.to_string();

    Some(InstalledApp {
        name,
        path: exec,
    })
}

#[cfg(target_os = "windows")]
fn collect_windows_apps(apps: &mut Vec<InstalledApp>) {
    let roots = [
        std::env::var("ProgramData")
            .ok()
            .map(|p| PathBuf::from(p).join("Microsoft/Windows/Start Menu/Programs")),
        std::env::var("APPDATA")
            .ok()
            .map(|p| PathBuf::from(p).join("Microsoft/Windows/Start Menu/Programs")),
    ];

    for root in roots.into_iter().flatten() {
        collect_windows_shortcuts(&root, apps);
    }
}

#[cfg(target_os = "windows")]
fn collect_windows_shortcuts(dir: &Path, apps: &mut Vec<InstalledApp>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_windows_shortcuts(&path, apps);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("lnk") {
            continue;
        }

        let name = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("Unknown")
            .to_string();

        let launch_path = crate::apps::windows_lnk::resolve_lnk_target(&path).unwrap_or(path);

        apps.push(InstalledApp {
            name,
            path: launch_path.to_string_lossy().to_string(),
        });
    }
}

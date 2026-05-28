use std::fs;
use std::path::{Path, PathBuf};

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct BrowserProfile {
    pub id: String,
    pub name: String,
}

pub fn supports_profiles(path: &str) -> bool {
    let lower = path.to_lowercase();
    if lower.contains("safari") {
        return false;
    }
    super::browser::is_browser(path)
}

pub fn list_profiles(app_path: &str) -> Vec<BrowserProfile> {
    if !supports_profiles(app_path) {
        return Vec::new();
    }

    let lower = app_path.to_lowercase();
    if is_firefox_family(&lower) {
        return list_firefox_profiles();
    }

    chromium_user_data_dir(app_path).map_or_else(Vec::new, |dir| list_chromium_profiles(&dir))
}

pub fn build_profile_args(_app_path: &str, profile_id: &str) -> Vec<String> {
    if profile_id.starts_with("firefox:") {
        let rel = profile_id.trim_start_matches("firefox:");
        if let Some(full) = resolve_firefox_profile_path(rel) {
            return vec![format!("--profile={}", full.display())];
        }
        return Vec::new();
    }

    vec![format!("--profile-directory={profile_id}")]
}

fn is_firefox_family(lower_path: &str) -> bool {
    lower_path.contains("firefox")
        || lower_path.contains("librewolf")
        || lower_path.contains("waterfox")
        || lower_path.contains("zen")
}

fn home_dir() -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        return std::env::var_os("USERPROFILE").map(PathBuf::from);
    }
    #[cfg(not(target_os = "windows"))]
    {
        return std::env::var_os("HOME").map(PathBuf::from);
    }
}

fn chromium_user_data_dir(app_path: &str) -> Option<PathBuf> {
    let home = home_dir()?;
    let lower = app_path.to_lowercase();

    #[cfg(target_os = "macos")]
    {
        let base = home.join("Library/Application Support");
        if lower.contains("arc") {
            return Some(base.join("Arc/User Data"));
        }
        if lower.contains("microsoft edge") || (lower.contains(" edge") && lower.contains("edge")) {
            return Some(base.join("Microsoft Edge"));
        }
        if lower.contains("brave") {
            return Some(base.join("BraveSoftware/Brave-Browser"));
        }
        if lower.contains("vivaldi") {
            return Some(base.join("Vivaldi"));
        }
        if lower.contains("opera") {
            return Some(base.join("com.operasoftware.Opera"));
        }
        if lower.contains("chromium") {
            return Some(base.join("Chromium"));
        }
        if lower.contains("chrome") {
            return Some(base.join("Google/Chrome"));
        }
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        let base = home.join("AppData/Local");
        if lower.contains("microsoft edge") || lower.contains(" msedge") || lower.ends_with("edge.exe") {
            return Some(base.join("Microsoft/Edge/User Data"));
        }
        if lower.contains("brave") {
            return Some(base.join("BraveSoftware/Brave-Browser/User Data"));
        }
        if lower.contains("vivaldi") {
            return Some(base.join("Vivaldi/User Data"));
        }
        if lower.contains("opera") {
            return Some(base.join("Opera Software/Opera Stable/User Data"));
        }
        if lower.contains("chromium") {
            return Some(base.join("Chromium/User Data"));
        }
        if lower.contains("chrome") {
            return Some(base.join("Google/Chrome/User Data"));
        }
        return None;
    }

    #[cfg(target_os = "linux")]
    {
        let base = home.join(".config");
        if lower.contains("microsoft edge") || lower.contains(" msedge") {
            return Some(base.join("microsoft-edge"));
        }
        if lower.contains("brave") {
            return Some(base.join("BraveSoftware/Brave-Browser"));
        }
        if lower.contains("vivaldi") {
            return Some(base.join("vivaldi"));
        }
        if lower.contains("opera") {
            return Some(base.join("opera"));
        }
        if lower.contains("chromium") {
            return Some(base.join("chromium"));
        }
        if lower.contains("chrome") {
            return Some(base.join("google-chrome"));
        }
        return None;
    }
}

fn list_chromium_profiles(user_data_dir: &Path) -> Vec<BrowserProfile> {
    let mut profiles = read_chromium_profiles_from_local_state(user_data_dir);
    if profiles.is_empty() {
        profiles = scan_chromium_profile_dirs(user_data_dir);
    }
    profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    profiles
}

fn read_chromium_profiles_from_local_state(user_data_dir: &Path) -> Vec<BrowserProfile> {
    let local_state = user_data_dir.join("Local State");
    let content = match fs::read_to_string(&local_state) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let Some(cache) = json
        .pointer("/profile/info_cache")
        .and_then(|value| value.as_object())
    else {
        return Vec::new();
    };

    let mut profiles = Vec::new();
    for (id, info) in cache {
        if !is_chromium_profile_dir(user_data_dir, id) {
            continue;
        }
        let name = info
            .get("name")
            .and_then(|value| value.as_str())
            .unwrap_or(id.as_str())
            .to_string();
        profiles.push(BrowserProfile {
            id: id.clone(),
            name,
        });
    }
    profiles
}

fn scan_chromium_profile_dirs(user_data_dir: &Path) -> Vec<BrowserProfile> {
    let entries = match fs::read_dir(user_data_dir) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let mut profiles = Vec::new();
    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let id = file_name.to_string_lossy().to_string();
        if !is_chromium_profile_id(&id) {
            continue;
        }
        if !entry.path().join("Preferences").is_file() {
            continue;
        }
        let name = read_chromium_profile_name(&entry.path()).unwrap_or_else(|| id.clone());
        profiles.push(BrowserProfile { id, name });
    }
    profiles
}

fn is_chromium_profile_dir(user_data_dir: &Path, id: &str) -> bool {
    is_chromium_profile_id(id) && user_data_dir.join(id).join("Preferences").is_file()
}

fn is_chromium_profile_id(id: &str) -> bool {
    id == "Default" || id.starts_with("Profile ")
}

fn read_chromium_profile_name(profile_dir: &Path) -> Option<String> {
    let content = fs::read_to_string(profile_dir.join("Preferences")).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    json.pointer("/profile/name")
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn firefox_profiles_root() -> Option<PathBuf> {
    let home = home_dir()?;
    #[cfg(target_os = "macos")]
    {
        return Some(home.join("Library/Application Support/Firefox"));
    }
    #[cfg(target_os = "windows")]
    {
        return Some(home.join("AppData/Roaming/Mozilla/Firefox"));
    }
    #[cfg(target_os = "linux")]
    {
        return Some(home.join(".mozilla/firefox"));
    }
}

fn list_firefox_profiles() -> Vec<BrowserProfile> {
    let root = match firefox_profiles_root() {
        Some(value) => value,
        None => return Vec::new(),
    };
    let ini_path = root.join("profiles.ini");
    let content = match fs::read_to_string(&ini_path) {
        Ok(value) => value,
        Err(_) => return Vec::new(),
    };

    let mut profiles = Vec::new();
    let mut current_name: Option<String> = None;
    let mut current_path: Option<String> = None;
    let mut current_is_relative = true;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.ends_with(']') {
            if let (Some(name), Some(path)) = (current_name.take(), current_path.take()) {
                if let Some(full) = resolve_firefox_profile_path_with_root(&root, &path, current_is_relative) {
                    profiles.push(BrowserProfile {
                        id: format!("firefox:{path}"),
                        name,
                    });
                    let _ = full;
                }
            }
            current_is_relative = true;
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            match key.trim() {
                "Name" => current_name = Some(value.trim().to_string()),
                "Path" => current_path = Some(value.trim().to_string()),
                "IsRelative" => current_is_relative = value.trim() != "0",
                _ => {}
            }
        }
    }

    if let (Some(name), Some(path)) = (current_name, current_path) {
        if resolve_firefox_profile_path_with_root(&root, &path, current_is_relative).is_some() {
            profiles.push(BrowserProfile {
                id: format!("firefox:{path}"),
                name,
            });
        }
    }

    profiles.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    profiles
}

fn resolve_firefox_profile_path(rel_or_abs: &str) -> Option<PathBuf> {
    let root = firefox_profiles_root()?;
    resolve_firefox_profile_path_with_root(&root, rel_or_abs, !Path::new(rel_or_abs).is_absolute())
}

fn resolve_firefox_profile_path_with_root(
    root: &Path,
    path: &str,
    is_relative: bool,
) -> Option<PathBuf> {
    let candidate = if is_relative {
        root.join(path)
    } else {
        PathBuf::from(path)
    };
    if candidate.is_dir() {
        Some(candidate)
    } else {
        None
    }
}

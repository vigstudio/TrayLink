use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

use super::AppConfig;

#[cfg(target_os = "windows")]
fn migrate_lnk_paths(config: &mut AppConfig) -> bool {
    let mut changed = false;
    for entry in config.apps.values_mut() {
        if !entry.path.to_ascii_lowercase().ends_with(".lnk") {
            continue;
        }
        let resolved = crate::apps::resolve_launch_path(&entry.path);
        let new_path = resolved.to_string_lossy().to_string();
        if new_path != entry.path {
            entry.path = new_path;
            changed = true;
        }
    }
    changed
}

const STORE_PATH: &str = "config.json";
const DEFAULTS: &str = include_str!("../../../config/defaults.json");

fn strip_legacy_seed_apps(config: &mut AppConfig) -> bool {
    let legacy = [
        ("obs", "C:/Program Files/obs-studio/bin/64bit/obs64.exe"),
        ("calculator", "Calculator"),
    ];
    let mut changed = false;

    for (key, path) in legacy {
        if config
            .apps
            .get(key)
            .is_some_and(|entry| entry.path == path && entry.url.is_none())
        {
            config.apps.remove(key);
            changed = true;
        }
    }

    changed
}

pub fn load_config(app: &AppHandle) -> Result<AppConfig, String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;

    let mut config: AppConfig = if let Some(value) = store.get("config") {
        serde_json::from_value(value).map_err(|e| e.to_string())?
    } else {
        let mut defaults: AppConfig =
            serde_json::from_str(DEFAULTS).map_err(|e| e.to_string())?;
        if defaults.token.is_empty() {
            defaults.token = Uuid::new_v4().to_string();
        }
        store
            .set("config", serde_json::to_value(&defaults).map_err(|e| e.to_string())?);
        store.save().map_err(|e| e.to_string())?;
        defaults
    };

    if config.token.is_empty() {
        config.token = Uuid::new_v4().to_string();
        save_config(app, &config)?;
    }

    if strip_legacy_seed_apps(&mut config) {
        save_config(app, &config)?;
    }

    if migrate_url_enabled(&mut config) {
        save_config(app, &config)?;
    }

    if migrate_legacy_hotkeys(&mut config) {
        save_config(app, &config)?;
    }

    if migrate_app_hotkey_field(&mut config) {
        save_config(app, &config)?;
    }

    if sanitize_app_hotkeys(&mut config) {
        save_config(app, &config)?;
    }

    #[cfg(target_os = "windows")]
    if migrate_lnk_paths(&mut config) {
        save_config(app, &config)?;
    }

    Ok(config)
}

fn migrate_url_enabled(config: &mut AppConfig) -> bool {
    let mut changed = false;
    for entry in config.apps.values_mut() {
        if entry.url.is_some() && !entry.url_enabled {
            entry.url_enabled = true;
            changed = true;
        }
    }
    changed
}

fn migrate_legacy_hotkeys(config: &mut AppConfig) -> bool {
    if config.hotkeys.is_empty() {
        return false;
    }

    let mut changed = false;
    for entry in config.hotkeys.drain().map(|(_, v)| v) {
        if entry.target_type == "app" {
            if let Some(app) = config.apps.get_mut(&entry.target_key) {
                if app.hotkeys.is_empty() {
                    app.hotkeys.push(crate::config::AppHotkeyBinding {
                        id: "open".to_string(),
                        name: "Mở app".to_string(),
                        accelerator: entry.accelerator,
                        action: "open".to_string(),
                        icon: None,
                    });
                    changed = true;
                }
            }
        }
    }

    changed
}

fn migrate_app_hotkey_field(config: &mut AppConfig) -> bool {
    let mut changed = false;
    for app in config.apps.values_mut() {
        if app.migrate_legacy_fields() {
            changed = true;
        }
    }
    changed
}

fn sanitize_app_hotkeys(config: &mut AppConfig) -> bool {
    let mut changed = false;
    for app in config.apps.values_mut() {
        for binding in &mut app.hotkeys {
            if let Some(normalized) = crate::launcher::hotkey::normalize_accelerator_string(&binding.accelerator) {
                if normalized != binding.accelerator {
                    binding.accelerator = normalized;
                    changed = true;
                }
            }
        }
    }
    changed
}

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store
        .set(
            "config",
            serde_json::to_value(config).map_err(|e| e.to_string())?,
        );
    store.save().map_err(|e| e.to_string())
}

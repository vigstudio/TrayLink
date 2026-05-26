use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

use super::AppConfig;

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

pub fn save_config(app: &AppHandle, config: &AppConfig) -> Result<(), String> {
    let store = app.store(STORE_PATH).map_err(|e| e.to_string())?;
    store
        .set(
            "config",
            serde_json::to_value(config).map_err(|e| e.to_string())?,
        );
    store.save().map_err(|e| e.to_string())
}

use tauri::AppHandle;
use tauri_plugin_store::StoreExt;
use uuid::Uuid;

use super::AppConfig;

const STORE_PATH: &str = "config.json";
const DEFAULTS: &str = include_str!("../../../config/defaults.json");

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

    Ok(config)
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

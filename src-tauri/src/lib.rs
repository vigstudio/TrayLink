mod api;
mod apps;
mod config;
mod launcher;
mod net;
mod remote_icons;
mod tls;
mod state;
mod tray;

use std::sync::Arc;

use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

use crate::api::server::{restart_server, start_server};
use crate::config::store::{load_config, save_config};
use crate::state::AppState;

#[tauri::command]
fn get_config(state: tauri::State<'_, Arc<AppState>>) -> config::AppConfig {
    state.config.read().unwrap().clone()
}

#[tauri::command]
fn update_config(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    config: config::AppConfig,
) -> Result<(), String> {
    {
        let mut cfg = state.config.write().unwrap();
        *cfg = config.clone();
    }
    save_config(&app, &config)?;
    Ok(())
}

#[tauri::command]
async fn restart_api_server(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    restart_server(app, state.inner().clone()).await
}

#[tauri::command]
fn get_request_logs(state: tauri::State<'_, Arc<AppState>>) -> Vec<state::LogEntry> {
    state.logs.read().unwrap().clone()
}

#[tauri::command]
fn regenerate_token(app: tauri::AppHandle, state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    let token = uuid::Uuid::new_v4().to_string();
    {
        let mut cfg = state.config.write().unwrap();
        cfg.token = token.clone();
        save_config(&app, &cfg)?;
    }
    Ok(token)
}

#[tauri::command]
fn set_autostart(app: tauri::AppHandle, state: tauri::State<'_, Arc<AppState>>, enabled: bool) -> Result<(), String> {
    sync_autostart(&app, enabled)?;

    {
        let mut cfg = state.config.write().unwrap();
        cfg.autostart = enabled;
        save_config(&app, &cfg)?;
    }

    Ok(())
}

fn sync_autostart(app: &tauri::AppHandle, enabled: bool) -> Result<(), String> {
    use tauri_plugin_autostart::ManagerExt;

    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
fn get_autostart_enabled(app: tauri::AppHandle) -> Result<bool, String> {
    use tauri_plugin_autostart::ManagerExt;
    app.autolaunch().is_enabled().map_err(|e| e.to_string())
}

#[tauri::command]
fn get_server_uptime(state: tauri::State<'_, Arc<AppState>>) -> i64 {
    state.uptime_seconds()
}

#[tauri::command]
fn test_open_app(state: tauri::State<'_, Arc<AppState>>, app_key: String) -> Result<String, String> {
    let apps = state.config.read().unwrap().apps.clone();
    launcher::open_app::open_app(&app_key, &apps, None).map_err(|err| err.to_string())?;
    Ok(format!("Đã mở app '{app_key}'"))
}

#[tauri::command]
fn get_app_icon(path: String) -> Option<String> {
    apps::get_app_icon_data_url(&path)
}

#[tauri::command]
fn set_deck_icon_from_file(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    item_type: String,
    key: String,
    source_path: String,
) -> Result<String, String> {
    let filename = remote_icons::save_custom_icon(&app, &source_path)?;
    let slot = remote_icons::slot_id(&item_type, &key);

    {
        let mut cfg = state.config.write().unwrap();
        if let Some(old) = cfg.remote_deck.custom_icons.insert(slot, filename.clone()) {
            let _ = remote_icons::delete_custom_icon(&app, &old);
        }
        save_config(&app, &cfg)?;
    }

    remote_icons::custom_icon_data_url(&app, &filename).ok_or_else(|| "Không đọc được icon".to_string())
}

#[tauri::command]
fn clear_deck_icon(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    item_type: String,
    key: String,
) -> Result<(), String> {
    let slot = remote_icons::slot_id(&item_type, &key);

    {
        let mut cfg = state.config.write().unwrap();
        if let Some(old) = cfg.remote_deck.custom_icons.remove(&slot) {
            remote_icons::delete_custom_icon(&app, &old)?;
        }
        save_config(&app, &cfg)?;
    }

    Ok(())
}

#[tauri::command]
fn get_deck_icon_data_url(
    app: tauri::AppHandle,
    state: tauri::State<'_, Arc<AppState>>,
    item_type: String,
    key: String,
) -> Option<String> {
    let cfg = state.config.read().unwrap();
    let slot = remote_icons::slot_id(&item_type, &key);
    let filename = cfg.remote_deck.custom_icons.get(&slot)?;
    remote_icons::custom_icon_data_url(&app, filename)
}

#[tauri::command]
fn list_installed_apps_cmd() -> Vec<apps::InstalledApp> {
    apps::list_installed_apps()
}

#[derive(serde::Serialize)]
struct ServerStatusResponse {
    online: bool,
    version: String,
    port: u16,
    https_port: u16,
    lan_ip: Option<String>,
    error: Option<String>,
}

#[tauri::command]
fn get_server_status(state: tauri::State<'_, Arc<AppState>>) -> ServerStatusResponse {
    let port = state.config.read().unwrap().port;
    ServerStatusResponse {
        online: state.is_server_running(),
        version: state::APP_VERSION.to_string(),
        port,
        https_port: api::server::https_port(port),
        lan_ip: net::get_lan_ip(),
        error: state.server_error.read().unwrap().clone(),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec![]),
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            tray::show_main_window(app);
        }))
        .setup(|app| {
            let config = load_config(app.handle())?;
            let state = Arc::new(AppState::new(config.clone(), app.handle().clone()));

            let _ = sync_autostart(app.handle(), config.autostart);

            app.manage(state.clone());

            tray::setup_tray(app.handle())?;
            tray::hide_from_dock(app.handle());

            if let Some(window) = app.get_webview_window("main") {
                let app_handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        tray::hide_main_window(&app_handle);
                    }
                });
            }

            let app_handle = app.handle().clone();
            let state_clone = state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = start_server(app_handle, state_clone).await {
                    eprintln!("Failed to start API server: {err}");
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_config,
            update_config,
            restart_api_server,
            get_request_logs,
            regenerate_token,
            set_autostart,
            get_autostart_enabled,
            get_server_uptime,
            get_server_status,
            list_installed_apps_cmd,
            get_app_icon,
            set_deck_icon_from_file,
            clear_deck_icon,
            get_deck_icon_data_url,
            test_open_app,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app_handle, event| match event {
            tauri::RunEvent::ExitRequested { api, code, .. } if code.is_none() => {
                api.prevent_exit();
                tray::hide_main_window(&app_handle);
            }
            #[cfg(target_os = "macos")]
            tauri::RunEvent::Reopen { .. } => {
                tray::show_main_window(&app_handle);
            }
            _ => {}
        });
}

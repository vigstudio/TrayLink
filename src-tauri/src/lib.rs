mod api;
mod apps;
mod config;
mod launcher;
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
    use tauri_plugin_autostart::ManagerExt;

    let autostart = app.autolaunch();
    if enabled {
        autostart.enable().map_err(|e| e.to_string())?;
    } else {
        autostart.disable().map_err(|e| e.to_string())?;
    }

    {
        let mut cfg = state.config.write().unwrap();
        cfg.autostart = enabled;
        save_config(&app, &cfg)?;
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
fn list_installed_apps_cmd() -> Vec<apps::InstalledApp> {
    apps::list_installed_apps()
}

#[derive(serde::Serialize)]
struct ServerStatusResponse {
    online: bool,
    version: String,
    port: u16,
    error: Option<String>,
}

#[tauri::command]
fn get_server_status(state: tauri::State<'_, Arc<AppState>>) -> ServerStatusResponse {
    let port = state.config.read().unwrap().port;
    ServerStatusResponse {
        online: state.is_server_running(),
        version: state::APP_VERSION.to_string(),
        port,
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
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let config = load_config(app.handle())?;
            let state = Arc::new(AppState::new(config, app.handle().clone()));

            if state.config.read().unwrap().autostart {
                use tauri_plugin_autostart::ManagerExt;
                let _ = app.autolaunch().enable();
            }

            app.manage(state.clone());

            tray::setup_tray(app.handle())?;

            if let Some(window) = app.get_webview_window("main") {
                let window_clone = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        let _ = window_clone.hide();
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

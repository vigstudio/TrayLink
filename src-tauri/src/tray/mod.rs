use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Manager,
};

use crate::api::server::restart_server;

pub fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let open_item = MenuItem::with_id(app, "open", "Open Dashboard", true, None::<&str>)?;
    let restart_item = MenuItem::with_id(app, "restart", "Restart Server", true, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, "quit", "Exit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open_item, &restart_item, &quit_item])?;

    let icon = app
        .default_window_icon()
        .ok_or("missing default window icon")?
        .clone();

    TrayIconBuilder::new()
        .icon(icon)
        .menu(&menu)
        .tooltip("TrayLink")
        .on_menu_event(|app, event| match event.id.as_ref() {
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            "restart" => {
                if let Some(state) = app.try_state::<std::sync::Arc<crate::state::AppState>>() {
                    let app_handle = app.clone();
                    let state_clone = state.inner().clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(err) = restart_server(app_handle, state_clone).await {
                            eprintln!("Failed to restart server: {err}");
                        }
                    });
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

use std::collections::HashMap;
use std::sync::mpsc;

use tauri::AppHandle;

use crate::config::{AppEntry, AppHotkeyBinding};
use crate::launcher::hotkey;
use crate::launcher::LauncherError;

/// macOS yêu cầu gửi phím (CGEvent) trên main thread — API/Remote chạy trên thread khác.
pub fn execute_binding(
    app_handle: &AppHandle,
    app_key: &str,
    binding: &AppHotkeyBinding,
    apps: &HashMap<String, AppEntry>,
) -> Result<(), LauncherError> {
    let (tx, rx) = mpsc::channel();
    let app_key = app_key.to_string();
    let binding = binding.clone();
    let apps = apps.clone();

    app_handle
        .run_on_main_thread(move || {
            let result = hotkey::execute_binding(&app_key, &binding, &apps);
            let _ = tx.send(result);
        })
        .map_err(|err| LauncherError::LaunchFailed(format!("main thread: {err}")))?;

    rx.recv()
        .map_err(|_| LauncherError::LaunchFailed("Không nhận được kết quả gửi phím".to_string()))?
}

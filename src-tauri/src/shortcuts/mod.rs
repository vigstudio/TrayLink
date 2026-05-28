use std::collections::HashMap;
use std::sync::Arc;

use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::config::AppConfig;
use crate::launcher::hotkey::{normalize_accelerator_string};
#[cfg(not(target_os = "macos"))]
use crate::launcher::hotkey;
use crate::state::AppState;
#[cfg(target_os = "macos")]
use crate::macos;

struct HotkeyTarget {
    app_key: String,
    binding_id: String,
}

pub fn setup_plugin(app: &AppHandle) -> Result<(), String> {
    let app_handle = app.clone();
    app.plugin(
        tauri_plugin_global_shortcut::Builder::new()
            .with_handler(move |app, shortcut, event| {
                if event.state() != ShortcutState::Pressed {
                    return;
                }
                let state = app.state::<Arc<AppState>>();
                if let Some(target) = lookup_hotkey(state.inner(), shortcut) {
                    let config = state.config.read().unwrap();
                    let apps = config.apps.clone();
                    if let Some(entry) = apps.get(&target.app_key) {
                        if let Some(binding) = entry
                            .hotkeys
                            .iter()
                            .find(|b| b.id == target.binding_id)
                        {
                            let result = {
                                #[cfg(target_os = "macos")]
                                {
                                    macos::executor::execute_binding(
                                        app,
                                        &target.app_key,
                                        binding,
                                        &apps,
                                    )
                                }
                                #[cfg(not(target_os = "macos"))]
                                {
                                    hotkey::execute_binding(&target.app_key, binding, &apps)
                                }
                            };
                            if let Err(err) = result {
                                eprintln!(
                                    "Hotkey '{}' / '{}': {err}",
                                    target.app_key, binding.name
                                );
                            }
                        }
                    }
                }
            })
            .build(),
    )
    .map_err(|e| e.to_string())?;

    if let Some(state) = app.try_state::<Arc<AppState>>() {
        let config = state.config.read().unwrap().clone();
        sync_hotkeys(&app_handle, &config);
    }

    Ok(())
}

pub fn sync_hotkeys(app: &AppHandle, config: &AppConfig) {
    let gs = app.global_shortcut();
    if let Err(err) = gs.unregister_all() {
        eprintln!("Không gỡ phím tắt cũ: {err}");
    }

    let mut seen = HashMap::new();
    for (app_key, entry) in &config.apps {
        for binding in &entry.hotkeys {
            // Phím "Gửi phím" (Save, Copy…) chỉ qua Remote/API — không đăng ký global
            // để tránh chặn phím tắt gốc của Photoshop và app khác.
            if binding.action != "open" {
                continue;
            }

            let Some(accelerator) = normalize_accelerator(&binding.accelerator) else {
                eprintln!(
                    "App '{app_key}' / '{}': bỏ qua phím tắt không hợp lệ '{}'",
                    binding.name, binding.accelerator
                );
                continue;
            };

            if seen.contains_key(&accelerator) {
                eprintln!(
                    "App '{app_key}' / '{}': trùng phím tắt '{accelerator}'",
                    binding.name
                );
                continue;
            }

            if let Err(err) = gs.register(accelerator.as_str()) {
                eprintln!(
                    "App '{app_key}' / '{}': không đăng ký được '{accelerator}': {err}",
                    binding.name
                );
                continue;
            }

            seen.insert(accelerator, (app_key.clone(), binding.id.clone()));
        }
    }
}

pub fn normalize_accelerator(raw: &str) -> Option<String> {
    let normalized = normalize_accelerator_string(raw)?;
    if normalized.parse::<Shortcut>().is_err() {
        return None;
    }
    Some(normalized)
}

fn lookup_hotkey(state: &AppState, shortcut: &Shortcut) -> Option<HotkeyTarget> {
    let config = state.config.read().unwrap();
    for (app_key, entry) in &config.apps {
        for binding in &entry.hotkeys {
            let Some(normalized) = normalize_accelerator(&binding.accelerator) else {
                continue;
            };
            if shortcut.matches_accelerator(&normalized) {
                return Some(HotkeyTarget {
                    app_key: app_key.clone(),
                    binding_id: binding.id.clone(),
                });
            }
        }
    }
    None
}

trait ShortcutMatch {
    fn matches_accelerator(&self, accelerator: &str) -> bool;
}

impl ShortcutMatch for Shortcut {
    fn matches_accelerator(&self, accelerator: &str) -> bool {
        if let Ok(parsed) = accelerator.parse::<Shortcut>() {
            return self == &parsed;
        }
        false
    }
}

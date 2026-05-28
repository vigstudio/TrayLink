mod discover;
mod icon;
mod windows_lnk;

use std::path::PathBuf;

pub use discover::{list_installed_apps, InstalledApp};
pub use icon::{get_app_icon_data_url, get_app_icon_png};

pub fn resolve_launch_path(path: &str) -> PathBuf {
    windows_lnk::resolve_launch_path(path)
}

#[cfg(target_os = "windows")]
pub use windows_lnk::{is_lnk, launch_windows_path};

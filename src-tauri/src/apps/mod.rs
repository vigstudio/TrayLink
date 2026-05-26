mod discover;
mod icon;

pub use discover::{list_installed_apps, InstalledApp};
pub use icon::{get_app_icon_data_url, get_app_icon_png};

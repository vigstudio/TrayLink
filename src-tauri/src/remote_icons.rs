use std::fs;
use std::path::{Path, PathBuf};

use base64::{engine::general_purpose::STANDARD, Engine as _};
use tauri::{AppHandle, Manager};
use uuid::Uuid;

const ICONS_SUBDIR: &str = "remote-icons";

pub fn icons_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| e.to_string())?
        .join(ICONS_SUBDIR);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn slot_id(item_type: &str, key: &str) -> String {
    format!("{item_type}:{key}")
}

pub fn icon_path(app: &AppHandle, filename: &str) -> Result<PathBuf, String> {
    Ok(icons_dir(app)?.join(filename))
}

pub fn save_custom_icon(app: &AppHandle, source_path: &str) -> Result<String, String> {
    let source = Path::new(source_path);
    if !source.is_file() {
        return Err("File không tồn tại".to_string());
    }

    let ext = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("png")
        .to_ascii_lowercase();

    if !matches!(ext.as_str(), "png" | "jpg" | "jpeg" | "webp" | "gif" | "svg") {
        return Err("Định dạng ảnh không được hỗ trợ".to_string());
    }

    let filename = format!("{}.{}", Uuid::new_v4(), ext);
    let dest = icon_path(app, &filename)?;
    fs::copy(source, &dest).map_err(|e| e.to_string())?;
    Ok(filename)
}

pub fn load_custom_icon(app: &AppHandle, filename: &str) -> Option<Vec<u8>> {
    let path = icon_path(app, filename).ok()?;
    fs::read(path).ok()
}

pub fn custom_icon_data_url(app: &AppHandle, filename: &str) -> Option<String> {
    let bytes = load_custom_icon(app, filename)?;
    let mime = mime_from_filename(filename);
    Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

pub fn custom_icon_filename(icon: &str) -> Option<&str> {
    icon.strip_prefix("custom:")
}

pub fn delete_hotkey_icon(app: &AppHandle, icon: Option<&str>) {
    if let Some(filename) = icon.and_then(custom_icon_filename) {
        let _ = delete_custom_icon(app, filename);
    }
}

pub fn delete_custom_icon(app: &AppHandle, filename: &str) -> Result<(), String> {
    let path = icon_path(app, filename)?;
    if path.exists() {
        fs::remove_file(path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn content_type_for_filename(filename: &str) -> &'static str {
    match Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        _ => "image/png",
    }
}

fn mime_from_filename(filename: &str) -> &'static str {
    content_type_for_filename(filename)
}

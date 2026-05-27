use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager};
use uuid::Uuid;

const UPLOADS_SUBDIR: &str = "TrayLink";
pub const MAX_UPLOAD_BYTES: u64 = 5 * 1024 * 1024 * 1024;

const ALLOWED_EXTENSIONS: &[&str] = &[
    // Images
    "png", "jpg", "jpeg", "gif", "webp", "heic", "heif", "bmp", "svg", "ico", "tiff", "tif",
    // Video
    "mp4", "mov", "avi", "mkv", "webm", "m4v", "3gp", "wmv", "flv",
    // Audio
    "mp3", "wav", "m4a", "aac", "flac", "ogg", "wma",
    // Documents
    "pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt", "md", "csv", "rtf", "odt", "ods",
    "odp", "pages", "numbers", "key",
    // Archives
    "zip", "rar", "7z", "tar", "gz",
];

pub fn uploads_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .download_dir()
        .map_err(|e| e.to_string())?
        .join(UPLOADS_SUBDIR);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

pub fn is_extension_allowed(filename: &str) -> bool {
    let ext = Path::new(filename)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    !ext.is_empty() && ALLOWED_EXTENSIONS.contains(&ext.as_str())
}

pub fn sanitize_filename(name: &str) -> String {
    let path = Path::new(name);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    let safe_stem: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .chars()
        .take(80)
        .collect();

    let stem = if safe_stem.is_empty() {
        "file".to_string()
    } else {
        safe_stem.replace(' ', "_")
    };

    let safe_ext: String = ext
        .chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .take(10)
        .collect();

    if safe_ext.is_empty() {
        stem
    } else {
        format!("{stem}.{safe_ext}")
    }
}

pub fn unique_dest_path(dir: &Path, original_name: &str) -> PathBuf {
    let sanitized = sanitize_filename(original_name);
    let path = Path::new(&sanitized);
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("file");
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");

    let base = format!("{stem}_{}", &Uuid::new_v4().simple().to_string()[..8]);
    let filename = if ext.is_empty() {
        base
    } else {
        format!("{base}.{ext}")
    };
    dir.join(filename)
}

pub fn save_upload(
    app: &AppHandle,
    original_name: &str,
    data: &[u8],
) -> Result<(PathBuf, String), String> {
    if data.is_empty() {
        return Err("File rỗng".to_string());
    }
    if data.len() as u64 > MAX_UPLOAD_BYTES {
        return Err(format!(
            "File quá lớn (tối đa {} GB)",
            MAX_UPLOAD_BYTES / 1024 / 1024 / 1024
        ));
    }
    if !is_extension_allowed(original_name) {
        return Err("Định dạng file không được hỗ trợ".to_string());
    }

    let dir = uploads_dir(app)?;
    let dest = unique_dest_path(&dir, original_name);
    let mut file = fs::File::create(&dest).map_err(|e| e.to_string())?;
    file.write_all(data).map_err(|e| e.to_string())?;

    let filename = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "unknown".to_string());

    Ok((dest, filename))
}

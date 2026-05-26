use std::path::{Path, PathBuf};
use std::process::Command;

use base64::{engine::general_purpose::STANDARD, Engine as _};

pub fn get_app_icon_png(path: &str) -> Option<Vec<u8>> {
    get_app_icon_data_url(path).and_then(|url| {
        url.strip_prefix("data:image/png;base64,")
            .and_then(|encoded| STANDARD.decode(encoded).ok())
    })
}

pub fn get_app_icon_data_url(path: &str) -> Option<String> {
    let resolved = resolve_app_path(path)?;

    #[cfg(target_os = "macos")]
    {
        return macos_icon_data_url(&resolved);
    }

    #[cfg(target_os = "windows")]
    {
        return windows_icon_data_url(&resolved);
    }

    #[cfg(target_os = "linux")]
    {
        return linux_icon_data_url(&resolved);
    }

    #[allow(unreachable_code)]
    None
}

fn resolve_app_path(path: &str) -> Option<PathBuf> {
    let candidate = PathBuf::from(path);
    if candidate.exists() {
        return Some(candidate);
    }

    #[cfg(target_os = "macos")]
    {
        for base in [
            PathBuf::from("/Applications"),
            PathBuf::from("/System/Applications"),
        ] {
            let app = base.join(format!("{path}.app"));
            if app.exists() {
                return Some(app);
            }
        }
        if let Ok(home) = std::env::var("HOME") {
            let app = PathBuf::from(home)
                .join("Applications")
                .join(format!("{path}.app"));
            if app.exists() {
                return Some(app);
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
fn macos_icon_data_url(app_path: &Path) -> Option<String> {
    if !app_path.extension().is_some_and(|ext| ext == "app") {
        return None;
    }

    let resources = app_path.join("Contents/Resources");
    let icns = std::fs::read_dir(resources).ok()?.flatten().find_map(|entry| {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "icns") {
            Some(path)
        } else {
            None
        }
    })?;

    let tmp = std::env::temp_dir().join(format!(
        "traylink-icon-{}.png",
        uuid::Uuid::new_v4()
    ));

    let status = Command::new("sips")
        .args([
            "-s",
            "format",
            "png",
            icns.to_str()?,
            "--out",
            tmp.to_str()?,
        ])
        .status()
        .ok()?;

    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }

    let bytes = std::fs::read(&tmp).ok()?;
    let _ = std::fs::remove_file(&tmp);
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(target_os = "windows")]
fn windows_icon_data_url(path: &Path) -> Option<String> {
    if !path.is_file() {
        return None;
    }

    let tmp = std::env::temp_dir().join(format!(
        "traylink-icon-{}.png",
        uuid::Uuid::new_v4()
    ));

    let script = format!(
        "Add-Type -AssemblyName System.Drawing; \
         $icon = [System.Drawing.Icon]::ExtractAssociatedIcon('{}'); \
         $bitmap = $icon.ToBitmap(); \
         $bitmap.Save('{}', [System.Drawing.Imaging.ImageFormat]::Png);",
        path.display(),
        tmp.display()
    );

    let status = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            &script,
        ])
        .status()
        .ok()?;

    if !status.success() {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }

    let bytes = std::fs::read(&tmp).ok()?;
    let _ = std::fs::remove_file(&tmp);
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

#[cfg(target_os = "linux")]
fn linux_icon_data_url(path: &Path) -> Option<String> {
    if path.extension().is_some_and(|ext| ext == "desktop") {
        if let Ok(content) = std::fs::read_to_string(path) {
            for line in content.lines() {
                if let Some(icon_name) = line.strip_prefix("Icon=") {
                    return find_linux_icon_file(icon_name.trim());
                }
            }
        }
    }

    if path.is_file() {
        return png_file_data_url(path);
    }

    None
}

#[cfg(target_os = "linux")]
fn find_linux_icon_file(name: &str) -> Option<String> {
    if name.starts_with('/') && Path::new(name).exists() {
        return png_file_data_url(Path::new(name));
    }

    for base in ["/usr/share/pixmaps", "/usr/share/icons/hicolor/48x48/apps"] {
        for ext in ["png", "svg"] {
            let candidate = PathBuf::from(base).join(format!("{name}.{ext}"));
            if candidate.exists() {
                if ext == "png" {
                    return png_file_data_url(&candidate);
                }
            }
        }
    }

    None
}

#[cfg(target_os = "linux")]
fn png_file_data_url(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    Some(format!("data:image/png;base64,{}", STANDARD.encode(bytes)))
}

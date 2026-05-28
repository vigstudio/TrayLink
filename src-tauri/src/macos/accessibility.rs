use core_foundation::base::TCFType;
use core_foundation::boolean::CFBoolean;
use core_foundation::dictionary::CFDictionary;
use core_foundation::string::CFString;
use serde::Serialize;
use std::os::raw::c_void;
use std::path::Path;
use std::process::Command;

pub const STABLE_BUNDLE_ID: &str = "com.phamminhkha.traylink";

#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXIsProcessTrustedWithOptions(options: *const c_void) -> bool;
}

#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityStatus {
    pub supported: bool,
    pub trusted: bool,
    pub dev_build: bool,
    pub executable_path: Option<String>,
    pub dev_app_path: Option<String>,
    pub codesign_identifier: Option<String>,
    pub stable_signature: bool,
    pub hint: String,
}

pub fn is_dev_executable(path: &str) -> bool {
    path.contains("/target/debug/") || path.contains("/target/release/")
}

pub fn codesign_identifier(path: &Path) -> Option<String> {
    let output = Command::new("codesign")
        .args(["-dv", path.to_str()?])
        .output()
        .ok()?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    for line in stderr.lines() {
        if let Some(rest) = line.strip_prefix("Identifier=") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

pub fn sign_binary(path: &Path, identifier: &str) -> Result<(), String> {
    let status = Command::new("codesign")
        .args(["--force", "--sign", "-", "--identifier", identifier])
        .arg(path)
        .status()
        .map_err(|err| err.to_string())?;

    if status.success() {
        Ok(())
    } else {
        Err(format!("codesign thất bại (exit {status})"))
    }
}

/// Ký lại binary dev với bundle ID cố định — giúp macOS giữ quyền Accessibility sau rebuild.
pub fn ensure_stable_dev_signature() {
    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(path_str) = exe.to_str() else {
        return;
    };
    if !is_dev_executable(path_str) {
        return;
    }

    let current_id = codesign_identifier(&exe);
    if current_id.as_deref() == Some(STABLE_BUNDLE_ID) {
        return;
    }

    if let Err(err) = sign_binary(&exe, STABLE_BUNDLE_ID) {
        eprintln!("Không ký được dev binary: {err}");
    } else {
        eprintln!(
            "Đã ký dev binary với {STABLE_BUNDLE_ID}. Quit TrayLink và mở lại để macOS nhận quyền."
        );
    }
}

pub fn sign_current_dev_binary() -> Result<String, String> {
    let exe = std::env::current_exe().map_err(|err| err.to_string())?;
    if !exe
        .to_str()
        .is_some_and(is_dev_executable)
    {
        return Err("Chỉ dùng khi chạy bản dev (target/debug).".to_string());
    }

    sign_binary(&exe, STABLE_BUNDLE_ID)?;
    Ok(format!(
        "Đã ký lại:\n{}\n\nQuit TrayLink hoàn toàn → bật Accessibility cho file trên → mở lại app.",
        exe.display()
    ))
}

pub fn status() -> AccessibilityStatus {
    let executable_path = std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string());
    let dev_build = executable_path
        .as_deref()
        .is_some_and(is_dev_executable);
    let codesign_identifier = std::env::current_exe()
        .ok()
        .as_ref()
        .and_then(|path| codesign_identifier(path));
    let stable_signature = codesign_identifier.as_deref() == Some(STABLE_BUNDLE_ID);
    let dev_app_path = dev_app_bundle_path();
    let trusted = is_trusted();

    let hint = if trusted {
        if dev_build {
            "Đã có quyền Accessibility cho bản dev hiện tại.".to_string()
        } else {
            "TrayLink đã có quyền Accessibility.".to_string()
        }
    } else if dev_build {
        format!(
            "macOS thường KHÔNG ghi nhận quyền khi chạy file traylink lẻ từ terminal.\n\n\
            Làm đúng thứ tự:\n\
            1. Quit TrayLink (cả terminal đang chạy tauri dev)\n\
            2. Chạy: npm run sign:dev\n\
            3. System Settings → Accessibility → bấm + → chọn:\n\
               TrayLink Dev.app (trong target/debug/)\n\
               KHÔNG chọn file traylink lẻ\n\
            4. Chạy: npm run dev:app\n\n\
            Nếu vẫn lỗi: bấm \"Reset quyền Accessibility\" rồi thêm lại TrayLink Dev.app.",
        )
    } else {
        "Bật TrayLink trong System Settings → Privacy & Security → Accessibility. \
         Nếu đã bật: tắt/bật lại toggle, Quit app và mở lại."
            .to_string()
    };

    AccessibilityStatus {
        supported: true,
        trusted,
        dev_build,
        executable_path,
        dev_app_path,
        codesign_identifier,
        stable_signature,
        hint,
    }
}

pub fn is_trusted() -> bool {
    unsafe { AXIsProcessTrusted() }
}

pub fn prompt_permission() -> bool {
    if is_trusted() {
        return true;
    }

    unsafe {
        let key = CFString::new("AXTrustedCheckOptionPrompt");
        let value = CFBoolean::true_value();
        let dict = CFDictionary::from_CFType_pairs(&[(key.as_CFType(), value.as_CFType())]);
        AXIsProcessTrustedWithOptions(dict.as_concrete_TypeRef() as *const c_void)
    }
}

pub fn dev_app_bundle_path() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let path_str = exe.to_str()?;
    if !is_dev_executable(path_str) {
        return None;
    }
    let debug_dir = exe.parent()?;
    let app = debug_dir.join("TrayLink Dev.app");
    if app.is_dir() {
        Some(app.display().to_string())
    } else {
        None
    }
}

pub fn open_dev_app_bundle() -> Result<String, String> {
    let app = dev_app_bundle_path()
        .ok_or_else(|| "Chưa có TrayLink Dev.app — chạy npm run sign:dev trước.".to_string())?;
    Command::new("open")
        .arg(&app)
        .spawn()
        .map_err(|err| err.to_string())?;
    Ok(format!("Đã mở {app}\nQuit bản traylink đang chạy từ terminal nếu có."))
}

pub fn reset_accessibility_approval() -> Result<String, String> {
    let output = Command::new("tccutil")
        .args(["reset", "Accessibility", STABLE_BUNDLE_ID])
        .output()
        .map_err(|err| format!("Không chạy được tccutil: {err}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(format!(
            "{stdout}\n\nĐã reset quyền Accessibility. Quit TrayLink → thêm lại TrayLink Dev.app → mở lại."
        ))
    } else {
        Err(if stderr.is_empty() { stdout } else { stderr })
    }
}

pub fn open_settings() -> Result<(), String> {
    Command::new("open")
        .arg("x-apple.systempreferences:com.apple.settings.PrivacySecurity.extension?Privacy_Accessibility")
        .spawn()
        .map_err(|err| err.to_string())?;
    Ok(())
}

pub fn input_permission_error() -> String {
    let exe = std::env::current_exe()
        .ok()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "(unknown)".to_string());
    let trusted = is_trusted();
    let id = std::env::current_exe()
        .ok()
        .as_ref()
        .and_then(|path| codesign_identifier(path))
        .unwrap_or_else(|| "(unknown)".to_string());
    let dev_app = dev_app_bundle_path().unwrap_or_else(|| "(chưa tạo — npm run sign:dev)".to_string());

    if is_dev_executable(&exe) {
        format!(
            "Không gửi được phím — macOS chưa cấp Accessibility (trusted={trusted}).\n\n\
             Khi dev, đừng bật quyền cho file traylink lẻ. Thêm app bundle:\n{dev_app}\n\n\
             1. npm run sign:dev\n\
             2. Quit TrayLink\n\
             3. Bật TrayLink Dev.app trong Accessibility\n\
             4. npm run dev:app"
        )
    } else if !trusted {
        format!(
            "Không gửi được phím — macOS chưa cấp Accessibility (trusted={trusted}).\n\
             Bật quyền cho:\n{exe}\n(ID: {id})\nSau khi bật: Quit hẳn TrayLink và mở lại."
        )
    } else {
        format!(
            "Không gửi được phím dù Accessibility đã bật.\nBinary: {exe}"
        )
    }
}

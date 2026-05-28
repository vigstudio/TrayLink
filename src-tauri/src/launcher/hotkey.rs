use crate::config::AppHotkeyBinding;
use crate::launcher::open_app::open_app;
use crate::launcher::LauncherError;
use std::collections::HashMap;
use crate::config::AppEntry;

#[cfg(any(target_os = "windows", target_os = "linux"))]
use std::{thread, time::Duration};

pub fn execute_binding(
    app_key: &str,
    binding: &AppHotkeyBinding,
    apps: &HashMap<String, AppEntry>,
) -> Result<(), LauncherError> {
    let entry = apps
        .get(app_key)
        .ok_or_else(|| LauncherError::AppNotAllowed(app_key.to_string()))?;

    if binding.action == "open" {
        return open_app(app_key, apps, None);
    }

    #[cfg(target_os = "macos")]
    {
        crate::macos::input::activate_and_focus(&entry.path)?;
    }

    #[cfg(any(target_os = "windows", target_os = "linux"))]
    {
        activate_app(&entry.path)?;
        thread::sleep(Duration::from_millis(600));
    }

    let accelerator = normalize_accelerator_string(&binding.accelerator)
        .unwrap_or_else(|| binding.accelerator.clone());
    send_accelerator(&accelerator).map_err(|err| {
        #[cfg(target_os = "macos")]
        if err.is_empty() || err.contains("permission") || err.contains("trusted") {
            return LauncherError::LaunchFailed(crate::macos::accessibility::input_permission_error());
        }
        LauncherError::LaunchFailed(err)
    })
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn activate_app(path: &str) -> Result<(), LauncherError> {
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;

        let launch_path = crate::apps::resolve_launch_path(path);
        Command::new(launch_path)
            .spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        return Ok(());
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;

        Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(LauncherError::LaunchFailed(
        "Kích hoạt app chưa hỗ trợ trên nền tảng này".to_string(),
    ))
}

pub fn send_accelerator(accelerator: &str) -> Result<(), String> {
    let normalized =
        normalize_accelerator_string(accelerator).unwrap_or_else(|| accelerator.to_string());
    let parsed = parse_accelerator(&normalized)?;

    #[cfg(target_os = "macos")]
    return crate::macos::input::send_parsed(&parsed);

    #[cfg(target_os = "windows")]
    return send_accelerator_enigo(&parsed);

    #[cfg(target_os = "linux")]
    return send_accelerator_enigo(&parsed);

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    Err("Gửi phím tắt chưa hỗ trợ trên nền tảng này".to_string())
}

pub(crate) struct ParsedAccelerator {
    pub command: bool,
    pub shift: bool,
    pub alt: bool,
    pub control: bool,
    pub key: KeySpec,
}

pub(crate) enum KeySpec {
    Char(char),
    Named(String),
}

pub fn normalize_accelerator_string(raw: &str) -> Option<String> {
    let parts: Vec<&str> = raw.split('+').map(str::trim).filter(|p| !p.is_empty()).collect();
    if parts.len() < 2 {
        return None;
    }

    let (modifiers, key_part) = parts.split_at(parts.len() - 1);
    if modifiers.is_empty() {
        return None;
    }

    let key = normalize_key_token(key_part[0])?;
    Some(format!("{}+{}", modifiers.join("+"), key))
}

fn normalize_key_token(token: &str) -> Option<String> {
    const SPECIAL: &[&str] = &[
        "Space", "Delete", "Backspace", "Enter", "Escape", "Tab", "Up", "Down", "Left", "Right",
        "Home", "End", "PageUp", "PageDown", "Insert", "F1", "F2", "F3", "F4", "F5", "F6", "F7",
        "F8", "F9", "F10", "F11", "F12",
    ];

    if SPECIAL.contains(&token) {
        return Some(token.to_string());
    }

    if token.len() == 1 {
        let ch = token.chars().next()?;
        if ch.is_ascii_alphanumeric() {
            return Some(ch.to_ascii_uppercase().to_string());
        }
        return latin_letter_to_ascii(ch);
    }

    None
}

fn latin_letter_to_ascii(ch: char) -> Option<String> {
    let mapped = match ch {
        'À' | 'Á' | 'Â' | 'Ã' | 'Ä' | 'Å' | 'Æ' | 'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' | 'æ' => 'A',
        'Ç' | 'ç' => 'C',
        'È' | 'É' | 'Ê' | 'Ë' | 'è' | 'é' | 'ê' | 'ë' => 'E',
        'Ì' | 'Í' | 'Î' | 'Ï' | 'ì' | 'í' | 'î' | 'ï' => 'I',
        'Ñ' | 'ñ' => 'N',
        'Ò' | 'Ó' | 'Ô' | 'Õ' | 'Ö' | 'Ø' | 'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => 'O',
        'Ù' | 'Ú' | 'Û' | 'Ü' | 'ù' | 'ú' | 'û' | 'ü' => 'U',
        'Ý' | 'Ÿ' | 'ý' | 'ÿ' => 'Y',
        'Đ' | 'đ' => 'D',
        _ => return None,
    };
    Some(mapped.to_string())
}

fn parse_accelerator(raw: &str) -> Result<ParsedAccelerator, String> {
    let parts: Vec<&str> = raw.split('+').map(str::trim).filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        return Err("Phím tắt không hợp lệ".to_string());
    }

    let mut command = false;
    let mut shift = false;
    let mut alt = false;
    let mut control = false;
    let mut key_part: Option<&str> = None;

    for part in parts {
        match part {
            "CommandOrControl" | "CmdOrCtrl" | "Command" | "Cmd" => command = true,
            "Control" | "Ctrl" => control = true,
            "Shift" => shift = true,
            "Alt" | "Option" => alt = true,
            other => {
                if key_part.is_some() {
                    return Err(format!("Phím tắt không hợp lệ: {raw}"));
                }
                key_part = Some(other);
            }
        }
    }

    let key_name = key_part.ok_or_else(|| format!("Thiếu phím trong '{raw}'"))?;
    let key = if key_name.len() == 1 {
        KeySpec::Char(key_name.chars().next().unwrap().to_ascii_lowercase())
    } else {
        KeySpec::Named(match key_name {
            "Space" => "Space".to_string(),
            "Delete" => "Delete".to_string(),
            "Backspace" => "Backspace".to_string(),
            "Enter" | "Return" => "Return".to_string(),
            "Escape" | "Esc" => "Escape".to_string(),
            "Tab" => "Tab".to_string(),
            "Up" => "Up".to_string(),
            "Down" => "Down".to_string(),
            "Left" => "Left".to_string(),
            "Right" => "Right".to_string(),
            "Home" => "Home".to_string(),
            "End" => "End".to_string(),
            "PageUp" => "PageUp".to_string(),
            "PageDown" => "PageDown".to_string(),
            other if other.starts_with('F') && other.len() <= 3 => other.to_string(),
            other => return Err(format!("Phím '{other}' chưa được hỗ trợ")),
        })
    };

    if !command && !shift && !alt && !control {
        return Err("Cần ít nhất một phím modifier (Ctrl/⌘, Shift, Alt)".to_string());
    }

    Ok(ParsedAccelerator {
        command,
        shift,
        alt,
        control,
        key,
    })
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn send_accelerator_enigo(parsed: &ParsedAccelerator) -> Result<(), String> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};

    let mut enigo = Enigo::new(&Settings::default()).map_err(|e| e.to_string())?;
    let modifiers = modifier_keys(parsed);

    for key in &modifiers {
        enigo.key(*key, Direction::Press).map_err(|e| e.to_string())?;
    }

    let click_result = match &parsed.key {
        KeySpec::Char(ch) => enigo.key(Key::Unicode(*ch), Direction::Click),
        KeySpec::Named(name) => {
            let key = named_key(name).ok_or_else(|| format!("Phím '{name}' chưa được hỗ trợ"))?;
            enigo.key(key, Direction::Click)
        }
    };
    click_result.map_err(|e| e.to_string())?;

    for key in modifiers.iter().rev() {
        enigo
            .key(*key, Direction::Release)
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn modifier_keys(parsed: &ParsedAccelerator) -> Vec<enigo::Key> {
    use enigo::Key;

    let mut keys = Vec::new();
    if parsed.command {
        keys.push(Key::Control);
    }
    if parsed.control {
        keys.push(Key::Control);
    }
    if parsed.shift {
        keys.push(Key::Shift);
    }
    if parsed.alt {
        keys.push(Key::Alt);
    }
    keys
}

#[cfg(any(target_os = "windows", target_os = "linux"))]
fn named_key(name: &str) -> Option<enigo::Key> {
    use enigo::Key;

    Some(match name {
        "Space" => Key::Space,
        "Delete" => Key::Delete,
        "Backspace" => Key::Backspace,
        "Return" => Key::Return,
        "Escape" => Key::Escape,
        "Tab" => Key::Tab,
        "Up" => Key::UpArrow,
        "Down" => Key::DownArrow,
        "Left" => Key::LeftArrow,
        "Right" => Key::RightArrow,
        "Home" => Key::Home,
        "End" => Key::End,
        "PageUp" => Key::PageUp,
        "PageDown" => Key::PageDown,
        "F1" => Key::F1,
        "F2" => Key::F2,
        "F3" => Key::F3,
        "F4" => Key::F4,
        "F5" => Key::F5,
        "F6" => Key::F6,
        "F7" => Key::F7,
        "F8" => Key::F8,
        "F9" => Key::F9,
        "F10" => Key::F10,
        "F11" => Key::F11,
        "F12" => Key::F12,
        _ => return None,
    })
}

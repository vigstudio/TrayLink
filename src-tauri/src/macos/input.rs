use std::process::Command;
use std::thread;
use std::time::Duration;

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

use crate::launcher::hotkey::{ParsedAccelerator, KeySpec};
use crate::launcher::LauncherError;

pub fn activate_and_focus(path: &str) -> Result<(), LauncherError> {
    let name = app_name_from_path(path);
    let script = format!("tell application {name:?} to activate");
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(LauncherError::LaunchFailed(if stderr.is_empty() {
            format!("Không kích hoạt được {name}")
        } else {
            stderr
        }));
    }

    wait_for_frontmost(&name);
    Ok(())
}

pub fn send_parsed(parsed: &ParsedAccelerator) -> Result<(), String> {
    if !crate::macos::accessibility::is_trusted() {
        let _ = crate::macos::accessibility::prompt_permission();
    }

    let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
        .map_err(|_| "Không tạo được nguồn sự kiện bàn phím (CGEventSource)".to_string())?;

    let key_code = key_spec_to_code(&parsed.key)?;
    let flags = modifier_flags(parsed);

    let key_down = CGEvent::new_keyboard_event(source.clone(), key_code, true)
        .map_err(|_| "Không tạo sự kiện key down".to_string())?;
    key_down.set_flags(flags);
    key_down.post(CGEventTapLocation::HID);

    thread::sleep(Duration::from_millis(20));

    let key_up = CGEvent::new_keyboard_event(source, key_code, false)
        .map_err(|_| "Không tạo sự kiện key up".to_string())?;
    key_up.set_flags(flags);
    key_up.post(CGEventTapLocation::HID);

    if !crate::macos::accessibility::is_trusted() {
        return Err(crate::macos::accessibility::input_permission_error());
    }

    Ok(())
}

fn wait_for_frontmost(app_name: &str) {
    let needle = app_name.to_lowercase();
    for _ in 0..30 {
        if let Ok(front) = frontmost_process_name() {
            let front_lower = front.to_lowercase();
            if front_lower.contains(&needle) || needle.contains(&front_lower) {
                thread::sleep(Duration::from_millis(150));
                return;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
    thread::sleep(Duration::from_millis(400));
}

fn frontmost_process_name() -> Result<String, String> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg(
            "tell application \"System Events\" to get name of first application process whose frontmost is true",
        )
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn app_name_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .and_then(|stem| stem.to_str())
        .filter(|name| !name.is_empty())
        .unwrap_or(path)
        .to_string()
}

fn modifier_flags(parsed: &ParsedAccelerator) -> CGEventFlags {
    let mut flags = CGEventFlags::CGEventFlagNull;
    if parsed.command {
        flags.insert(CGEventFlags::CGEventFlagCommand);
    }
    if parsed.shift {
        flags.insert(CGEventFlags::CGEventFlagShift);
    }
    if parsed.alt {
        flags.insert(CGEventFlags::CGEventFlagAlternate);
    }
    if parsed.control {
        flags.insert(CGEventFlags::CGEventFlagControl);
    }
    flags
}

fn key_spec_to_code(key: &KeySpec) -> Result<CGKeyCode, String> {
    match key {
        KeySpec::Char(ch) => char_key_code(*ch)
            .ok_or_else(|| format!("Phím '{ch}' chưa được hỗ trợ trên bàn phím US")),
        KeySpec::Named(name) => named_key_code(name)
            .ok_or_else(|| format!("Phím '{name}' chưa được hỗ trợ")),
    }
}

fn char_key_code(ch: char) -> Option<CGKeyCode> {
    Some(match ch.to_ascii_lowercase() {
        'a' => 0x00,
        'b' => 0x0B,
        'c' => 0x08,
        'd' => 0x02,
        'e' => 0x0E,
        'f' => 0x03,
        'g' => 0x05,
        'h' => 0x04,
        'i' => 0x22,
        'j' => 0x26,
        'k' => 0x28,
        'l' => 0x25,
        'm' => 0x2E,
        'n' => 0x2D,
        'o' => 0x1F,
        'p' => 0x23,
        'q' => 0x0C,
        'r' => 0x0F,
        's' => 0x01,
        't' => 0x11,
        'u' => 0x20,
        'v' => 0x09,
        'w' => 0x0D,
        'x' => 0x07,
        'y' => 0x10,
        'z' => 0x06,
        '0' => 0x1D,
        '1' => 0x12,
        '2' => 0x13,
        '3' => 0x14,
        '4' => 0x15,
        '5' => 0x17,
        '6' => 0x16,
        '7' => 0x1A,
        '8' => 0x1C,
        '9' => 0x19,
        _ => return None,
    })
}

fn named_key_code(name: &str) -> Option<CGKeyCode> {
    Some(match name {
        "Space" => 49,
        "Delete" => 51,
        "Return" => 36,
        "Escape" => 53,
        "Tab" => 48,
        "Up" => 126,
        "Down" => 125,
        "Left" => 123,
        "Right" => 124,
        "Home" => 115,
        "End" => 119,
        "PageUp" => 116,
        "PageDown" => 121,
        "F1" => 122,
        "F2" => 120,
        "F3" => 99,
        "F4" => 118,
        "F5" => 96,
        "F6" => 97,
        "F7" => 98,
        "F8" => 100,
        "F9" => 101,
        "F10" => 109,
        "F11" => 103,
        "F12" => 111,
        _ => return None,
    })
}

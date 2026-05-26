use std::collections::HashMap;
use std::process::Command;

use crate::config::ExecEntry;
use super::LauncherError;

pub fn exec_command(cmd_key: &str, commands: &HashMap<String, ExecEntry>) -> Result<bool, LauncherError> {
    let entry = commands
        .get(cmd_key)
        .ok_or_else(|| LauncherError::CommandNotAllowed(cmd_key.to_string()))?;

    if entry.internal {
        return Ok(true);
    }

    let shell_cmd = platform_command(entry).ok_or_else(|| {
        LauncherError::CommandNotAllowed(format!("{cmd_key} not configured for this platform"))
    })?;

    run_shell_command(&shell_cmd)?;
    Ok(false)
}

fn platform_command(entry: &ExecEntry) -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        return entry.win.clone();
    }
    #[cfg(target_os = "macos")]
    {
        return entry.mac.clone();
    }
    #[cfg(target_os = "linux")]
    {
        return entry.linux.clone();
    }
    #[allow(unreachable_code)]
    None
}

fn run_shell_command(command: &str) -> Result<(), LauncherError> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", command])
            .spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .spawn()
            .map_err(|e| LauncherError::LaunchFailed(e.to_string()))?;
    }

    Ok(())
}

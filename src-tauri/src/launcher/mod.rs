pub mod browser;
pub mod exec_cmd;
pub mod hotkey;
pub mod open_app;
pub mod open_file;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum LauncherError {
    #[error("app not found in allowlist: {0}")]
    AppNotAllowed(String),
    #[error("command not found in whitelist: {0}")]
    CommandNotAllowed(String),
    #[error("path not allowed: {0}")]
    PathNotAllowed(String),
    #[error("path does not exist: {0}")]
    PathNotFound(String),
    #[error("failed to launch: {0}")]
    LaunchFailed(String),
}

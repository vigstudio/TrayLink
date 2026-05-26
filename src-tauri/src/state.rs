use std::sync::{Arc, RwLock};
use std::time::Instant;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tokio::sync::{Mutex, oneshot};

use crate::config::AppConfig;

pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const MAX_LOG_ENTRIES: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub method: String,
    pub path: String,
    pub status: u16,
    pub duration_ms: u64,
    pub client_ip: String,
}

pub struct ServerHandle {
    pub shutdown_tx: Option<oneshot::Sender<()>>,
}

pub struct AppState {
    pub config: RwLock<AppConfig>,
    pub logs: RwLock<Vec<LogEntry>>,
    pub server: Mutex<Option<ServerHandle>>,
    pub app_handle: AppHandle,
    pub started_at: Instant,
    pub server_started_at: RwLock<Option<Instant>>,
    pub server_error: RwLock<Option<String>>,
}

impl AppState {
    pub fn new(config: AppConfig, app_handle: AppHandle) -> Self {
        Self {
            config: RwLock::new(config),
            logs: RwLock::new(Vec::new()),
            server: Mutex::new(None),
            app_handle,
            started_at: Instant::now(),
            server_started_at: RwLock::new(None),
            server_error: RwLock::new(None),
        }
    }

    pub fn uptime_seconds(&self) -> i64 {
        if let Some(started) = *self.server_started_at.read().unwrap() {
            return started.elapsed().as_secs() as i64;
        }
        self.started_at.elapsed().as_secs() as i64
    }

    pub fn is_server_running(&self) -> bool {
        self.server_started_at.read().unwrap().is_some()
            && self.server_error.read().unwrap().is_none()
    }

    pub fn push_log(&self, entry: LogEntry) {
        let mut logs = self.logs.write().unwrap();
        logs.push(entry);
        if logs.len() > MAX_LOG_ENTRIES {
            let drain = logs.len() - MAX_LOG_ENTRIES;
            logs.drain(0..drain);
        }
    }
}

pub type SharedState = Arc<AppState>;

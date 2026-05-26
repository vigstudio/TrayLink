pub mod store;

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

fn default_allow_get() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub port: u16,
    pub token: String,
    #[serde(default)]
    pub require_token: bool,
    #[serde(default = "default_allow_get")]
    pub allow_get: bool,
    pub autostart: bool,
    pub apps: HashMap<String, AppEntry>,
    pub commands: HashMap<String, ExecEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub path: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecEntry {
    #[serde(default)]
    pub internal: bool,
    #[serde(default)]
    pub win: Option<String>,
    #[serde(default)]
    pub mac: Option<String>,
    #[serde(default)]
    pub linux: Option<String>,
}

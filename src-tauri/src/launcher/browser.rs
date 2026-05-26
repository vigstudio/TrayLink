const BROWSER_HINTS: &[&str] = &[
    "chrome",
    "chromium",
    "firefox",
    "safari",
    "edge",
    "brave",
    "opera",
    "arc",
    "vivaldi",
    "zen",
    "browser",
    "tor",
    "waterfox",
    "librewolf",
];

pub fn is_browser(path: &str) -> bool {
    let lower = path.to_lowercase();
    BROWSER_HINTS.iter().any(|hint| lower.contains(hint))
}

pub fn validate_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Ok(());
    }
    if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        Ok(())
    } else {
        Err("URL phải bắt đầu bằng http:// hoặc https://".to_string())
    }
}

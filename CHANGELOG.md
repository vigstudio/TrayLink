# Changelog

Tất cả thay đổi đáng chú ý của dự án TrayLink được ghi tại đây.

Format dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
và dự án tuân theo [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-05-26

### Added

- Ứng dụng Tauri 2 + Rust + React + shadcn/ui chạy nền với system tray
- HTTP REST API trên `127.0.0.1:8765` (port cấu hình được):
  - `GET /status` — health check
  - `POST /open-app` — mở app theo allowlist (hỗ trợ `url` cho trình duyệt)
  - `POST /open-file` — mở file bằng app mặc định OS
  - `POST /exec` — chạy command key đã whitelist
- Xác thực Bearer token / `X-API-Token` cho các endpoint POST
- Allowlist apps và command whitelist lưu bằng `tauri-plugin-store`
- Launcher cross-platform (Windows / macOS / Linux)
- System tray: Open Dashboard, Restart Server, Exit
- Ẩn window khi đóng — app chạy nền trên tray
- Autostart khi boot (bật/tắt từ Dashboard)
- Single instance — mở lại app sẽ focus Dashboard
- Dashboard 4 tab: Overview, Request Log, Apps & Commands, Settings
- Chọn app từ danh sách app đã cài hoặc duyệt file (dialog)
- Hỗ trợ URL mặc định cho trình duyệt (Chrome, Safari, Firefox, Edge, …)
- Request log ring buffer (~200 entries)
- README hướng dẫn cài đặt và sử dụng API

### Security

- API chỉ bind `127.0.0.1` (localhost)
- Chỉ mở app/command có trong allowlist
- `open-file` chặn path traversal và system paths
- `exec` chỉ chấp nhận command key, không chạy raw shell

[0.1.0]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.0

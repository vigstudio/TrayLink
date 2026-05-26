# Changelog

Tất cả thay đổi đáng chú ý của dự án TrayLink được ghi tại đây.

Format dựa trên [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
và dự án tuân theo [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] - 2026-05-26

### Added

- **Remote Deck** — mở link trên điện thoại/tablet cùng Wi‑Fi để điều khiển PC dạng grid icon kiểu Stream Deck
- Trang mobile `remote.html`: grid app/command, chạm để mở app trên PC
- API remote: `GET /`, `GET /remote`, `GET /api/deck`, `GET /api/icons/{kind}/{key}`
- Tab **Remote Deck** trên Dashboard: xem trước, sắp xếp thứ tự, ẩn/hiện app và command
- Kéo-thả sắp xếp bằng `@dnd-kit` (drag handle ≡), tự lưu khi thả; nút ↑/↓ làm fallback
- Icon tùy chỉnh cho từng ô deck (chọn file / khôi phục mặc định)
- Modal QR code để mở Remote Deck nhanh trên điện thoại
- Toolbar mobile: toàn màn hình (Fullscreen API + chế độ immersive cho iOS Safari), giữ màn hình sáng (Wake Lock)
- Icon SVG mặc định khi app chưa có PNG (tránh lỗi 404 trên mobile)
- Cấu hình `remote_deck`: `display_order`, `app_order`, `command_order`, `hidden_apps`, `hidden_commands`, `custom_icons`
- Overview hiển thị link Remote Deck và nút copy/QR

### Changed

- README bổ sung hướng dẫn Remote Deck
- Allowlist editor đồng bộ layout remote deck khi thêm/xóa app hoặc command

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

[0.1.6]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.6
[0.1.0]: https://github.com/PhamMinhKha/TrayLink/releases/tag/v0.1.0

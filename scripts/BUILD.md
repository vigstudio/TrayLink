# Hướng dẫn build TrayLink

## Yêu cầu

- Node.js 18+
- Rust 1.88+ (`rustup`)
- **macOS:** Xcode Command Line Tools
- **Windows:** Visual Studio Build Tools + WebView2

## Build nhanh

### macOS (trên máy Mac)

```bash
npm run build:macos
```

Build universal (Intel + Apple Silicon):

```bash
npm run build:macos:universal
```

Output: `release/macos/`
- `TrayLink.app`
- `TrayLink_x.x.x_universal.dmg` (hoặc tương tự)

Kiến trúc riêng:

```bash
bash scripts/build-macos.sh arm64   # Apple Silicon
bash scripts/build-macos.sh x64     # Intel
bash scripts/build-macos.sh universal
```

GitHub Actions build cả 3 bản (universal, arm64, x64) song song.

### Windows (trên máy Windows)

```powershell
npm run build:windows
```

Output: `release/windows/`
- `nsis/TrayLink_x.x.x_x64-setup.exe` — installer
- `msi/` — MSI installer (nếu có)

### Tự chọn nền tảng

```bash
npm run build:all
```

## Build Windows từ macOS

Không thể build `.exe` trực tiếp trên Mac. Dùng **GitHub Actions**:

1. Push code hoặc tạo tag `v0.1.0`
2. Vào **Actions → Build Release → Run workflow**
3. Tải artifact `traylink-windows` và `traylink-macos`

## Autostart khi boot

TrayLink dùng `tauri-plugin-autostart`:

| OS | Cơ chế |
|---|---|
| **macOS** | LaunchAgent (`~/Library/LaunchAgents/`) |
| **Windows** | Registry `HKCU\...\Run` |

**Cách bật:** Dashboard → Settings → **Autostart khi boot**

Sau khi cài app (không chạy dev mode), bật autostart và restart máy để kiểm tra.

### Lưu ý macOS

- Cài app vào `/Applications` từ file `.dmg` hoặc copy `TrayLink.app`
- Autostart hoạt động tốt nhất với bản build release, không phải `tauri dev`
- Nếu gặp *“Apple could not verify TrayLink is free of malware”*: app chưa được Apple ký — xem [README — macOS Gatekeeper](../README.md#macos-cảnh-báo-apple-could-not-verify)
- Nếu distribute rộng rãi, nên code-sign và notarize app

### Lưu ý Windows

- Dùng installer NSIS để cài vào `%LOCALAPPDATA%` hoặc Program Files
- Autostart trỏ tới đường dẫn `.exe` sau khi cài

## Icon

```bash
npm run icon
```

Sinh lại toàn bộ icon từ `public/icon.png`.

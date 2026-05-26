#!/usr/bin/env bash
# Build TrayLink for macOS (.app + .dmg)
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "==> TrayLink build (macOS)"
echo "Root: $ROOT_DIR"

if [[ "$(uname -s)" != "Darwin" ]]; then
  echo "Script này chỉ chạy trên macOS."
  exit 1
fi

if ! command -v npm >/dev/null; then
  echo "Thiếu npm. Cài Node.js trước."
  exit 1
fi

if ! command -v cargo >/dev/null; then
  echo "Thiếu Rust/cargo. Cài rustup trước."
  exit 1
fi

echo "==> Cài dependencies"
npm ci

if [[ -f "public/icon.png" ]]; then
  echo "==> Generate icons từ public/icon.png"
  npm run tauri icon public/icon.png
fi

TARGET="${1:-native}"
BUILD_ARGS=()

case "$TARGET" in
  native)
    echo "==> Build cho kiến trúc hiện tại ($(uname -m))"
    ;;
  universal)
    echo "==> Build universal binary (Intel + Apple Silicon)"
    rustup target add aarch64-apple-darwin x86_64-apple-darwin 2>/dev/null || true
    BUILD_ARGS+=(--target universal-apple-darwin)
    ;;
  arm64)
    echo "==> Build Apple Silicon (aarch64)"
    rustup target add aarch64-apple-darwin 2>/dev/null || true
    BUILD_ARGS+=(--target aarch64-apple-darwin)
    ;;
  x64)
    echo "==> Build Intel (x86_64)"
    rustup target add x86_64-apple-darwin 2>/dev/null || true
    BUILD_ARGS+=(--target x86_64-apple-darwin)
    ;;
  *)
    echo "Usage: $0 [native|universal|arm64|x64]"
    exit 1
    ;;
esac

echo "==> Tauri build (release)"
set +e
if ((${#BUILD_ARGS[@]} > 0)); then
  npm run tauri build -- "${BUILD_ARGS[@]}"
else
  npm run tauri build
fi
BUILD_EXIT=$?
set -e

BUNDLE_DIR="$ROOT_DIR/src-tauri/target/release/bundle/macos"
if [[ ! -d "$BUNDLE_DIR" ]]; then
  BUNDLE_DIR="$ROOT_DIR/src-tauri/target/universal-apple-darwin/release/bundle/macos"
fi
if [[ ! -d "$BUNDLE_DIR" ]]; then
  BUNDLE_DIR="$ROOT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/macos"
fi
if [[ ! -d "$BUNDLE_DIR" ]]; then
  BUNDLE_DIR="$ROOT_DIR/src-tauri/target/x86_64-apple-darwin/release/bundle/macos"
fi

if [[ ! -d "$BUNDLE_DIR/TrayLink.app" ]]; then
  echo "Build thất bại — không tìm thấy TrayLink.app"
  exit "${BUILD_EXIT:-1}"
fi

if [[ "$BUILD_EXIT" -ne 0 ]]; then
  echo "⚠️  Cảnh báo: bundling .dmg có thể thất bại, nhưng TrayLink.app đã build xong."
fi

RELEASE_DIR="$ROOT_DIR/release/macos"
mkdir -p "$RELEASE_DIR"

echo "==> Copy artifacts -> $RELEASE_DIR"
cp -R "$BUNDLE_DIR/TrayLink.app" "$RELEASE_DIR"/

DMG_DIR="$ROOT_DIR/src-tauri/target/release/bundle/dmg"
if [[ ! -d "$DMG_DIR" ]]; then
  DMG_DIR="$ROOT_DIR/src-tauri/target/universal-apple-darwin/release/bundle/dmg"
fi
if [[ ! -d "$DMG_DIR" ]]; then
  DMG_DIR="$ROOT_DIR/src-tauri/target/aarch64-apple-darwin/release/bundle/dmg"
fi
if [[ ! -d "$DMG_DIR" ]]; then
  DMG_DIR="$ROOT_DIR/src-tauri/target/x86_64-apple-darwin/release/bundle/dmg"
fi
if [[ -d "$DMG_DIR" ]]; then
  find "$DMG_DIR" -maxdepth 1 -name "*.dmg" ! -name "rw.*" -exec cp {} "$RELEASE_DIR"/ \; 2>/dev/null || true
fi

if [[ "$BUILD_EXIT" -ne 0 && ! -f "$RELEASE_DIR"/*.dmg ]]; then
  echo "Tip: cài create-dmg nếu cần file .dmg: brew install create-dmg"
fi

exit 0

echo
echo "✅ Build macOS xong!"
echo "Output:"
ls -la "$RELEASE_DIR"
echo
echo "Autostart: bật trong Dashboard -> Settings -> Autostart khi boot"
echo "  macOS: dùng LaunchAgent (tauri-plugin-autostart)"
echo "Cài đặt: mở file .dmg, kéo TrayLink.app vào Applications"

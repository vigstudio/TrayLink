#!/usr/bin/env bash
# Build TrayLink — tự chọn nền tảng hoặc build cả hai qua CI
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

OS="$(uname -s)"

echo "==> TrayLink build-all"
echo "Platform: $OS"

case "$OS" in
  Darwin)
    bash "$ROOT_DIR/scripts/build-macos.sh" "${1:-universal}"
    ;;
  MINGW*|MSYS*|CYGWIN*|Windows*)
    powershell -ExecutionPolicy Bypass -File "$ROOT_DIR/scripts/build-windows.ps1"
    ;;
  *)
    echo "Không hỗ trợ build trực tiếp trên $OS."
    echo "Dùng GitHub Actions: .github/workflows/build-release.yml"
    exit 1
    ;;
esac

echo
echo "Tip: để build Windows từ macOS, push code lên GitHub — workflow sẽ build cả 2 nền tảng."

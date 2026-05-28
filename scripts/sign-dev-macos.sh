#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
BINARY="${1:-$ROOT/src-tauri/target/debug/traylink}"
IDENTIFIER="com.phamminhkha.traylink"

if [[ ! -f "$BINARY" ]]; then
  echo "Chưa có binary: $BINARY (chạy cargo build trước)"
  exit 1
fi

codesign --force --sign - --identifier "$IDENTIFIER" "$BINARY"
echo "Đã ký dev binary với ID ổn định: $IDENTIFIER"
echo "  → $BINARY"

if [[ -x "$(dirname "$0")/sync-dev-app.sh" ]]; then
  bash "$(dirname "$0")/sync-dev-app.sh"
fi

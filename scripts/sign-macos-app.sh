#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

BUILD_META="$(
  node -e '
    const fs = require("fs");
    const config = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
    console.log(config.productName);
  '
)"

APP_PATH="${1:-$ROOT_DIR/src-tauri/target/release/bundle/macos/${BUILD_META}.app}"
SIGNING_IDENTITY="${APPLE_SIGNING_IDENTITY:--}"

if [[ ! -d "$APP_PATH" ]]; then
  echo "App bundle not found: $APP_PATH" >&2
  exit 1
fi

echo "Signing app bundle with identity: $SIGNING_IDENTITY"
codesign --force --deep --sign "$SIGNING_IDENTITY" "$APP_PATH"
codesign --verify --deep --strict --verbose=2 "$APP_PATH"

if [[ "$SIGNING_IDENTITY" == "-" ]]; then
  echo "Warning: ad-hoc signing was used. Some macOS machines may still refuse to launch the app." >&2
  echo "Set APPLE_SIGNING_IDENTITY to a valid code-signing identity for a launchable distribution build." >&2
fi

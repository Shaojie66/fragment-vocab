#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

BUILD_META="$(
  node -e '
    const fs = require("fs");
    const config = JSON.parse(fs.readFileSync("src-tauri/tauri.conf.json", "utf8"));
    const archMap = { arm64: "aarch64", x64: "x86_64" };
    console.log([config.productName, config.version, archMap[process.arch] || process.arch].join("\n"));
  '
)"

PRODUCT_NAME="$(printf '%s\n' "$BUILD_META" | sed -n '1p')"
VERSION="$(printf '%s\n' "$BUILD_META" | sed -n '2p')"
ARCH="$(printf '%s\n' "$BUILD_META" | sed -n '3p')"

APP_PATH="$ROOT_DIR/src-tauri/target/release/bundle/macos/${PRODUCT_NAME}.app"
DMG_DIR="$ROOT_DIR/src-tauri/target/release/bundle/dmg"
DMG_PATH="$DMG_DIR/${PRODUCT_NAME}_${VERSION}_${ARCH}.dmg"

echo "Building app bundle..."
npm run tauri build

if [[ ! -d "$APP_PATH" ]]; then
  echo "App bundle not found: $APP_PATH" >&2
  exit 1
fi

"$ROOT_DIR/scripts/sign-macos-app.sh" "$APP_PATH"

mkdir -p "$DMG_DIR"
rm -f "$DMG_PATH"

STAGING_DIR="$(mktemp -d)"
trap 'rm -rf "$STAGING_DIR"' EXIT

cp -R "$APP_PATH" "$STAGING_DIR/"
ln -s /Applications "$STAGING_DIR/Applications"

echo "Creating DMG..."
hdiutil create \
  -volname "$PRODUCT_NAME" \
  -srcfolder "$STAGING_DIR" \
  -ov \
  -format UDZO \
  "$DMG_PATH"

echo "Created DMG at: $DMG_PATH"

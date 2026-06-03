#!/usr/bin/env bash
# build-android-debug.sh
# 构建 Android debug APK（自带 debug keystore，不需要外部签名）。
# 用法：./scripts/build-android-debug.sh
# 产物：src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk

set -euo pipefail
cd "$(dirname "$0")/.."

echo "==> Android debug build"
cargo tauri android build --debug --ci

APK="src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk"
if [[ -f "$APK" ]]; then
  echo "✓ APK built: $(realpath "$APK")"
  ls -lh "$APK"
else
  echo "✗ APK not found at $APK — check build log"
  exit 1
fi

#!/usr/bin/env bash
# build-release.sh
# 在配置好的 Android 工具链上构建可发布 APK。
# 用法：./scripts/build-release.sh
# 产物：src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk

set -euo pipefail

# 环境要求（CI 中需要预装）：
# - Rust 1.93+ with targets: aarch64-linux-android, armv7-linux-androideabi, x86_64-linux-android
# - Android SDK: build-tools;34.0.0+, platforms;android-34
# - Android NDK r25+
# - JDK 17
# - ANDROID_HOME, ANDROID_NDK_HOME

cd "$(dirname "$0")/.."

echo "==> 1/3  Release build of native binaries"
cargo build --release -p williw-api-server -p williw-core

echo "==> 2/3  Release build of Tauri desktop shell (sanity check)"
cargo build --release -p williw-tauri

echo "==> 3/3  Release build of Android APK"
cargo tauri android build --ci

APK="src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk"
if [[ -f "$APK" ]]; then
  echo ""
  echo "✓ APK built: $(realpath "$APK")"
  ls -lh "$APK"
else
  echo "✗ APK not found at $APK — check build log"
  exit 1
fi

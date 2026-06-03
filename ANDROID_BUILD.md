# Android 构建与发布说明

本文档补充 README 中"Android 发布构建"一节，覆盖签名、版本、产物路径。

## 工具链版本

| 工具 | 最低版本 | 备注 |
|---|---|---|
| Rust | 1.93 | `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android` |
| Android SDK | build-tools 34+, platforms android-34 | `sdkmanager "build-tools;34.0.0" "platforms;android-34"` |
| Android NDK | r25 | `sdkmanager "ndk;25.1.8937393"` |
| JDK | 17 | Tauri 2 要求 17+ |
| tauri-cli | 2.9+ | `cargo install tauri-cli --version "^2.9"` |

## 一键脚本

```bash
./scripts/build-android-debug.sh    # 调试 APK（debug keystore）
./scripts/build-release.sh          # 发布 APK（需要在 android/gradle.properties 配置签名）
./scripts/test-e2e.sh               # 端到端测试（HTTP API）
```

## Debug 构建

- 自动使用 Android 默认 debug keystore（在 `~/.android/debug.keystore`）
- `AndroidManifest.xml` 中 `usesCleartextTraffic=true`（允许 127.0.0.1 HTTP 流量）
- 不需要任何额外配置

```bash
cargo tauri android build --debug --ci
# 产物：src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk
```

## Release 构建

需要在 `src-tauri/gen/android/app/build.gradle.kts` 中显式配置 release 签名（默认无签名）。
示例（请把 keystore 放在安全位置，**不要提交到仓库**）：

```kotlin
android {
    signingConfigs {
        create("release") {
            storeFile = file(System.getenv("ANDROID_KEYSTORE_PATH"))
            storePassword = System.getenv("ANDROID_KEYSTORE_PASSWORD")
            keyAlias = System.getenv("ANDROID_KEY_ALIAS")
            keyPassword = System.getenv("ANDROID_KEY_PASSWORD")
        }
    }
    buildTypes {
        getByName("release") {
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = false
        }
    }
}
```

## 安装到设备

```bash
adb install -r src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk
adb shell am start -n com.williw.app/.MainActivity
```

## 验证 API

```bash
# 从设备本地
adb shell curl http://127.0.0.1:8081/v1/models

# 从开发机（端口通过 adb reverse）
adb reverse tcp:8081 tcp:8081
curl http://127.0.0.1:8081/v1/chat/completions -H 'content-type: application/json' \
  -d '{"model":"williw-local","messages":[{"role":"user","content":"hi"}]}'
```

## 把模型推到设备

```bash
adb push models/qwen2.5-0.5b-instruct-q4_k_m.gguf /sdcard/Android/data/com.williw.app/files/models/
adb push models/tokenizer.json /sdcard/Android/data/com.williw.app/files/models/
```

应用首次启动会从 `filesDir/models/` 加载模型。

## 故障排查

| 现象 | 原因 | 处理 |
|---|---|---|
| `Missing tool clang.cmd` | NDK 是占位包，无 toolchain | `sdkmanager "ndk;25.1.8937393"` |
| `Could not find build tools` | 缺 build-tools | `sdkmanager "build-tools;34.0.0"` |
| `package android-34 not found` | 缺 platforms | `sdkmanager "platforms;android-34"` |
| 启动后 `/v1/status` 报 model missing | 模型没在 filesDir | 见上"把模型推到设备" |
| 推理报 `engine error` | 启用 candle 但未带 feature | `cargo build --features williw-core/candle` |

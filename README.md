# Williw — Local AI on Your Phone

> 0.1.0 MVP: 在安卓手机上加载最小可对话的小语言模型（默认 Qwen2.5-0.5B-Instruct Q4_K_M, ~350MB），
> 通过一个 OpenAI 兼容的本地 HTTP API 暴露给设备上的其他 App 或局域网。

## 当前状态（MVP）

| 能力 | 状态 |
|---|---|
| 加载 GGUF Qwen2.5 量化模型 | ✅（feature = `candle`，默认 mock 后端） |
| OpenAI 兼容 HTTP API | ✅（`/v1/chat/completions`、`/v1/models`、`/health`） |
| Tauri 2 桌面壳 | ✅（Windows/macOS/Linux 编译通过） |
| Tauri 2 Android 移动壳 | ✅（项目已初始化，可发布 APK 构建脚本就绪） |
| candle 后端实推理 | ✅（代码完整；首次启用需 `cargo build --features candle -p williw-core`） |

## 架构

```
┌──────────────────────────────────────────┐
│ Tauri 2 窗口（控制面板 / 聊天 UI）         │  ← frontend/dist/index.html
│ ┌──────────────────────────────────────┐ │
│ │ Axum HTTP API（端口 8081，0.0.0.0）   │ │  ← src-tauri/src/main.rs
│ │   - GET  /v1/models                  │ │
│ │   - POST /v1/chat/completions        │ │
│ │   - GET  /v1/status                  │ │
│ │   - GET  /health                     │ │
│ └──────────────┬───────────────────────┘ │
│                │                          │
│         williw-core::Engine               │  ← core/src/lib.rs
│           ├── MockEngine（默认）           │
│           └── GgufEngine（candle 特性）    │  ← core/src/gguf_engine.rs
└──────────────────────────────────────────┘
```

## 快速开始（开发机）

```bash
cd williw
# 1. 编译并运行 API 服务（mock 后端，无需任何模型文件）
cargo run -p williw-api-server --release

# 2. 测试
curl -X POST http://127.0.0.1:8081/v1/chat/completions \
  -H 'content-type: application/json' \
  -d '{"model":"williw-local","messages":[{"role":"user","content":"hi"}]}'
```

## 在 Tauri 桌面壳里跑

```bash
cargo run -p williw-tauri --release
# 弹出窗口，端口由 setup 回调自动注入
```

## 启用真模型推理

1. 下载 Qwen2.5-0.5B-Instruct Q4_K_M：
   - 模型：`https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct-GGUF/resolve/main/qwen2.5-0.5b-instruct-q4_k_m.gguf` (~350MB)
   - tokenizer：`https://huggingface.co/Qwen/Qwen2.5-0.5B-Instruct/resolve/main/tokenizer.json`
2. 放到 `williw/models/`（或任意位置并设置 `WILLIW_MODEL_DIR`）
3. 用 candle 特性重新编译：
   ```bash
   cargo build --release -p williw-api-server -p williw-tauri --features williw-core/candle
   ```
4. 启动时设环境变量：
   ```bash
   WILLIW_BACKEND=candle ./target/release/williw-api
   ```

> 首次 candle 编译会比较慢（10+ 分钟）；编译产物缓存后增量构建快。

## Android 发布构建

要求：
- Rust 1.93+
- Android SDK（build-tools 34+, platforms android-34）
- Android NDK r25+
- JDK 17
- 设置 `ANDROID_HOME` 与 `ANDROID_NDK_HOME`

```bash
cd williw
# 调试 APK（自带 debug keystore，免签名）
./scripts/build-android-debug.sh

# 发布 APK（需要在 gradle.properties 中配置 release 签名）
./scripts/build-release.sh
```

产物：
- 调试：`src-tauri/gen/android/app/build/outputs/apk/debug/app-debug.apk`
- 发布：`src-tauri/gen/android/app/build/outputs/apk/release/app-release.apk`

## HTTP API

### `GET /health`
```json
"ok"
```

### `GET /v1/models`
OpenAI 兼容。MVP 只注册一个本地模型。

### `GET /v1/status`
```json
{
  "state": "ready",
  "model_id": "qwen2.5-0.5b-instruct-q4_k_m",
  "model_path": "/path/to/qwen2.5-0.5b-instruct-q4_k_m.gguf",
  "context_len": 2048,
  "error": null,
  "last_prompt_tokens": 12,
  "last_completion_tokens": 87,
  "last_total_ms": 1234
}
```

### `POST /v1/chat/completions`
OpenAI 兼容，支持 `temperature` / `top_p` / `max_tokens` / `stop` / `stream`。
需要鉴权时设置 `WILLIW_API_KEY` 环境变量，客户端发 `Authorization: Bearer <key>`。

## 测试

```bash
cargo test --workspace          # 单元测试
bash scripts/test-e2e.sh        # 端到端冒烟测试
```

## 模型目录约定

- 桌面端：`./models/`（相对 cwd）或 `WILLIW_MODEL_DIR` 环境变量
- Android 端：`filesDir/models/`（由 `ANDROID_FILES_DIR` 推断）或 `WILLIW_MODEL_DIR` 覆盖

需要的文件：
- `qwen2.5-0.5b-instruct-q4_k_m.gguf`
- `tokenizer.json`

## License

MIT

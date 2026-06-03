//! Williw — Tauri 2 桌面 + Android 入口（v0.1.0）
//!
//! 启动顺序：
//! 1. 加载 settings.json（settings 持久化）
//! 2. 在后台 tokio runtime 启动 Axum HTTP API（端口 8081）
//! 3. 启动 Tauri 应用，把 Engine / Settings 注入 Tauri::State，
//!    前端通过 `invoke('cmd_xxx')` 调用，并通过 `window.__WILLIW_API_BASE__`
//!    拿到 API 端点。
//!
//! 端点：
//! - HTTP  : /health, /v1/models, /v1/status, /v1/chat/completions
//! - Cmd   : cmd_status, cmd_settings_get, cmd_settings_set,
//!           cmd_models_list, cmd_pick_model, cmd_unload, cmd_reload,
//!           cmd_download (MVP 占位), cmd_app_info

#![cfg_attr(mobile, tauri::mobile_entry_point)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

use axum::{
    Json, Router,
    extract::State as AxumState,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State as TauriState};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use williw_core::{ChatMessage, Engine, EngineConfig, EngineState, GenerateRequest};

// ===== 持久化设置 =====

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
struct AppSettings {
    api_port: u16,
    api_key: Option<String>,
    model_dir: Option<String>,
    default_model: Option<String>,
    temperature: f32,
    max_tokens: u32,
    top_p: f32,
    system_prompt: Option<String>,
    allow_external_access: bool,
    theme: String, // "dark" | "light" | "auto"
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            api_port: 8081,
            api_key: None,
            model_dir: None,
            default_model: None,
            temperature: 0.7,
            max_tokens: 512,
            top_p: 0.9,
            system_prompt: Some("You are Williw, a helpful local AI assistant running on the user's device.".into()),
            allow_external_access: false,
            theme: "dark".into(),
        }
    }
}

fn settings_path(app: &AppHandle) -> PathBuf {
    // 桌面端：~/.config/williw/settings.json 或 EXE 同目录
    // 移动端：通过 app.path().app_config_dir() 拿
    if let Ok(p) = app.path().app_config_dir() {
        return p.join("settings.json");
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("settings.json")
}

fn load_settings(path: &PathBuf) -> AppSettings {
    if let Ok(bytes) = std::fs::read(path) {
        if let Ok(s) = serde_json::from_slice::<AppSettings>(&bytes) {
            return s;
        }
    }
    AppSettings::default()
}

fn save_settings(path: &PathBuf, s: &AppSettings) -> std::io::Result<()> {
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let bytes = serde_json::to_vec_pretty(s).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    std::fs::write(path, bytes)
}

// ===== HTTP 共享状态 =====

#[derive(Clone)]
struct ApiAppState {
    engine: Arc<Engine>,
    model_list: Vec<ModelInfo>,
    api_key: Option<String>,
    settings: Arc<Mutex<AppSettings>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelInfo {
    id: String,
    object: String,
    created: i64,
    owned_by: String,
}

#[derive(Debug, Serialize)]
struct ModelsResponse { object: String, data: Vec<ModelInfo> }

#[derive(Debug, Deserialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(default)] max_tokens: Option<u32>,
    #[serde(default)] temperature: Option<f32>,
    #[serde(default)] top_p: Option<f32>,
    #[serde(default)] stop: Option<Vec<String>>,
    #[serde(default)] stream: Option<bool>,
}

#[derive(Debug, Serialize)]
struct ChatCompletionsResponse {
    id: String,
    object: String,
    created: i64,
    model: String,
    choices: Vec<Choice>,
    usage: Usage,
}

#[derive(Debug, Serialize)]
struct Choice {
    index: u32,
    message: ChatMessage,
    finish_reason: String,
}

#[derive(Debug, Serialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
struct ApiError { error: ApiErrorBody }
#[derive(Debug, Serialize)]
struct ApiErrorBody { message: String, #[serde(rename = "type")] kind: String }
impl ApiError {
    fn new(msg: impl Into<String>, kind: &str) -> Self {
        Self { error: ApiErrorBody { message: msg.into(), kind: kind.to_string() } }
    }
}
impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = serde_json::to_string(&self).unwrap_or_else(|_| r#"{"error":"unknown"}"#.into());
        (StatusCode::INTERNAL_SERVER_ERROR, [("content-type", "application/json")], body).into_response()
    }
}

#[derive(Debug, Serialize, Clone)]
struct StatusPayload {
    state: String,
    model_id: Option<String>,
    model_path: Option<String>,
    context_len: u32,
    error: Option<String>,
    last_prompt_tokens: Option<u32>,
    last_completion_tokens: Option<u32>,
    last_total_ms: Option<u64>,
}

// ===== 后台启动 Axum =====

fn spawn_api_server_blocking(state: ApiAppState) -> Result<u16, Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(2)
        .build()?;
    let port = state.settings.lock().api_port;
    let bound = rt.block_on(async move { spawn_api_server(state, port).await })?;
    // 让 runtime 持续跑（detach via leak）
    std::mem::forget(rt);
    Ok(bound)
}

async fn spawn_api_server(state: ApiAppState, port: u16) -> Result<u16, Box<dyn std::error::Error>> {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    let app = Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/status", get(status_route))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let bind: SocketAddr = if std::env::var("WILLIW_API_ALLOW_EXTERNAL").is_ok() {
        ([0, 0, 0, 0], port).into()
    } else {
        ([127, 0, 0, 1], port).into()
    };
    let listener = TcpListener::bind(bind).await?;
    let bound = listener.local_addr()?.port();
    tracing::info!("williw-api-server listening on http://{bind}");
    tokio::spawn(async move {
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("axum server error: {e}");
        }
    });
    Ok(bound)
}

fn model_dir_for(settings: &AppSettings) -> PathBuf {
    if let Some(p) = settings.model_dir.as_ref() {
        return PathBuf::from(p);
    }
    if let Ok(p) = std::env::var("WILLIW_MODEL_DIR") {
        return PathBuf::from(p);
    }
    #[cfg(target_os = "android")]
    {
        if let Some(dir) = std::env::var_os("ANDROID_FILES_DIR") {
            return PathBuf::from(dir).join("models");
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("models")
}

// ===== Tauri 入口 =====

struct AppContext {
    engine: Arc<Engine>,
    settings: Arc<Mutex<AppSettings>>,
    settings_path: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 1) 日志
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,williw_core=info,williw_tauri=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    tauri::Builder::default()
        .setup(|app| {
            // 解析 settings 路径
            let s_path = settings_path(&app.handle());
            let settings = load_settings(&s_path);
            tracing::info!("settings loaded from {} (api_port={})", s_path.display(), settings.api_port);

            // 构造 Engine
            let model_dir = model_dir_for(&settings);
            tracing::info!("model dir: {}", model_dir.display());
            let engine = Engine::new();
            engine.configure(EngineConfig::default_for_dir(model_dir));
            if let Err(e) = engine.load() {
                tracing::warn!("engine load returned: {e} (continuing, see /v1/status)");
            }

            // 把 Engine/Settings 注入 Tauri State
            let settings_arc = Arc::new(Mutex::new(settings.clone()));
            app.manage(AppContext {
                engine: engine.clone(),
                settings: settings_arc.clone(),
                settings_path: s_path.clone(),
            });

            // 启动 API server（后台 tokio runtime）
            let api_state = ApiAppState {
                engine: engine.clone(),
                model_list: vec![ModelInfo {
                    id: engine.status().model_id.clone().unwrap_or_else(|| "williw-local".into()),
                    object: "model".into(),
                    created: chrono::Utc::now().timestamp(),
                    owned_by: "williw".into(),
                }],
                api_key: settings.api_key.clone(),
                settings: settings_arc.clone(),
            };
            let port = match spawn_api_server_blocking(api_state) {
                Ok(p) => p,
                Err(e) => {
                    tracing::error!("api server failed to start: {e}");
                    0
                }
            };
            tracing::info!("API server bound on port {port}");

            // 注入 API base 到前端
            if let Some(window) = app.get_webview_window("main") {
                if port > 0 {
                    let _ = window.eval(&format!(
                        "window.__WILLIW_API_BASE__ = 'http://127.0.0.1:{port}'; window.__WILLIW_API_PORT__ = {port};"
                    ));
                }
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_app_info,
            cmd_status,
            cmd_settings_get,
            cmd_settings_set,
            cmd_models_list,
            cmd_unload,
            cmd_reload,
            cmd_download,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(not(mobile))]
fn main() { run(); }

// ===== Tauri Commands =====

#[derive(Debug, Serialize)]
struct AppInfo {
    version: String,
    name: String,
    api_port: u16,
    api_base: String,
    platform: &'static str,
    started_at_ms: u64,
}

fn started_at() -> u64 {
    // 进程启动时间——用 Instant::now 转化不可行，改用 chrono::Utc::now
    chrono::Utc::now().timestamp_millis() as u64
}

#[tauri::command]
fn cmd_app_info(state: TauriState<'_, AppContext>) -> AppInfo {
    let s = state.settings.lock();
    let port = s.api_port;
    drop(s);
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "Williw".to_string(),
        api_port: port,
        api_base: format!("http://127.0.0.1:{port}"),
        platform: std::env::consts::OS,
        started_at_ms: started_at(),
    }
}

#[derive(Debug, Serialize)]
struct CmdStatus {
    state: String,
    model_id: Option<String>,
    model_path: Option<String>,
    context_len: u32,
    error: Option<String>,
    last_prompt_tokens: Option<u32>,
    last_completion_tokens: Option<u32>,
    last_total_ms: Option<u64>,
    default_model: Option<String>,
    api_port: u16,
    model_dir: String,
}

#[tauri::command]
fn cmd_status(state: TauriState<'_, AppContext>) -> CmdStatus {
    let st = state.engine.status();
    let s = state.settings.lock();
    CmdStatus {
        state: format!("{:?}", st.state).to_lowercase(),
        model_id: st.model_id.clone(),
        model_path: st.model_path,
        context_len: st.context_len,
        error: st.error,
        last_prompt_tokens: st.last_prompt_tokens,
        last_completion_tokens: st.last_completion_tokens,
        last_total_ms: st.last_total_ms,
        default_model: s.default_model.clone().or_else(|| st.model_id.clone()),
        api_port: s.api_port,
        model_dir: model_dir_for(&s).display().to_string(),
    }
}

#[tauri::command]
fn cmd_settings_get(state: TauriState<'_, AppContext>) -> AppSettings {
    state.settings.lock().clone()
}

#[tauri::command]
fn cmd_settings_set(
    app: AppHandle,
    new_settings: AppSettings,
    state: TauriState<'_, AppContext>,
) -> Result<AppSettings, String> {
    *state.settings.lock() = new_settings.clone();
    save_settings(&state.settings_path, &new_settings).map_err(|e| e.to_string())?;
    tracing::info!("settings saved to {}", state.settings_path.display());
    let _ = app;
    Ok(new_settings)
}

#[derive(Debug, Serialize)]
struct ModelEntry {
    id: String,
    name: String,
    path: Option<String>,
    size_bytes: Option<u64>,
    state: String,
    is_default: bool,
    backend: String, // "mock" or "candle"
}

#[tauri::command]
fn cmd_models_list(state: TauriState<'_, AppContext>) -> Vec<ModelEntry> {
    let dir = model_dir_for(&state.settings.lock());
    let st = state.engine.status();
    let backend = std::env::var("WILLIW_BACKEND").unwrap_or_else(|_| "mock".into());
    let default_id = state.settings.lock().default_model.clone();
    let mut out = Vec::new();

    // 当前已加载的模型（来自 engine）
    if let Some(id) = st.model_id.clone() {
        let path = st.model_path.as_ref().map(PathBuf::from);
        let size = path.as_ref().and_then(|p| std::fs::metadata(p).ok()).map(|m| m.len());
        out.push(ModelEntry {
            id: id.clone(),
            name: id.clone(),
            path: path.map(|p| p.display().to_string()),
            size_bytes: size,
            state: format!("{:?}", st.state).to_lowercase(),
            is_default: default_id.as_deref() == Some(&id) || default_id.is_none(),
            backend: backend.clone(),
        });
    }

    // 扫描模型目录中的 .gguf 文件作为可下载/可加载候选项
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) == Some("gguf") {
                let id = p.file_stem().and_then(|s| s.to_str()).unwrap_or("model").to_string();
                if out.iter().any(|m| m.id == id) { continue; }
                let size = std::fs::metadata(&p).ok().map(|m| m.len());
                out.push(ModelEntry {
                    id: id.clone(),
                    name: id.clone(),
                    path: Some(p.display().to_string()),
                    size_bytes: size,
                    state: "not_loaded".into(),
                    is_default: default_id.as_deref() == Some(&id),
                    backend: backend.clone(),
                });
            }
        }
    }
    out
}

#[tauri::command]
fn cmd_unload(state: TauriState<'_, AppContext>) -> Result<(), String> {
    state.engine.unload();
    Ok(())
}

#[derive(Debug, Deserialize)]
struct ReloadRequest {
    model_path: Option<String>,
}

#[tauri::command]
fn cmd_reload(req: ReloadRequest, state: TauriState<'_, AppContext>) -> Result<CmdStatus, String> {
    let s = state.settings.lock().clone();
    let mut cfg = EngineConfig::default_for_dir(model_dir_for(&s));
    if let Some(p) = req.model_path {
        cfg.model_path = PathBuf::from(p);
    }
    state.engine.unload();
    state.engine.configure(cfg);
    state.engine.load().map_err(|e| e.to_string())?;
    Ok(cmd_status(state))
}

#[derive(Debug, Deserialize)]
struct DownloadRequest {
    url: String,
    filename: String,
}

#[derive(Debug, Serialize)]
struct DownloadResponse {
    ok: bool,
    message: String,
    saved_to: Option<String>,
}

#[tauri::command]
fn cmd_download(req: DownloadRequest, state: TauriState<'_, AppContext>) -> DownloadResponse {
    let dir = model_dir_for(&state.settings.lock());
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return DownloadResponse { ok: false, message: format!("create dir: {e}"), saved_to: None };
    }
    // MVP 占位：真下载需要 reqwest + 异步 + 进度回调。MVP 中我们返回友好提示，
    // 让用户用浏览器/curl 下载后放到模型目录——这是诚实做法。
    DownloadResponse {
        ok: false,
        message: format!(
            "MVP does not bundle a downloader. Please download the file manually:\n  curl -L '{}' -o '{}'\nThen it will appear in: {}",
            req.url,
            dir.join(&req.filename).display(),
            dir.display()
        ),
        saved_to: None,
    }
}

// ===== HTTP handlers =====

async fn health() -> &'static str { "ok" }

async fn list_models(AxumState(s): AxumState<ApiAppState>) -> Json<ModelsResponse> {
    Json(ModelsResponse { object: "list".into(), data: s.model_list.clone() })
}

async fn status_route(AxumState(s): AxumState<ApiAppState>) -> Json<StatusPayload> {
    let st = s.engine.status();
    Json(StatusPayload {
        state: format!("{:?}", st.state).to_lowercase(),
        model_id: st.model_id,
        model_path: st.model_path,
        context_len: st.context_len,
        error: st.error,
        last_prompt_tokens: st.last_prompt_tokens,
        last_completion_tokens: st.last_completion_tokens,
        last_total_ms: st.last_total_ms,
    })
}

async fn chat_completions(
    AxumState(s): AxumState<ApiAppState>,
    headers: axum::http::HeaderMap,
    Json(req): Json<ChatCompletionsRequest>,
) -> Response {
    if let Some(expected) = s.api_key.as_ref() {
        let provided = headers.get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "));
        if provided.map(|p| p == expected).unwrap_or(false) == false {
            return ApiError::new("invalid or missing API key", "authentication_error").into_response();
        }
    }
    if req.messages.is_empty() {
        return ApiError::new("messages must not be empty", "invalid_request_error").into_response();
    }
    let _started = Instant::now();
    let gen_req = GenerateRequest {
        messages: req.messages.clone(),
        max_tokens: req.max_tokens.unwrap_or(256),
        temperature: req.temperature.unwrap_or(0.7),
        top_p: req.top_p.unwrap_or(0.9),
        stop: req.stop.clone().unwrap_or_default(),
    };
    let engine = s.engine.clone();
    let result = tokio::task::spawn_blocking(move || {
        // 简单同步路径——async 包装其实更好但 MVP 同步够用
        engine.generate_blocking(gen_req)
    }).await;

    match result {
        Ok(Ok(r)) => {
            let resp = ChatCompletionsResponse {
                id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                object: "chat.completion".into(),
                created: chrono::Utc::now().timestamp(),
                model: req.model,
                choices: vec![Choice {
                    index: 0,
                    message: ChatMessage { role: "assistant".into(), content: r.text },
                    finish_reason: "stop".into(),
                }],
                usage: Usage {
                    prompt_tokens: r.prompt_tokens,
                    completion_tokens: r.completion_tokens,
                    total_tokens: r.prompt_tokens + r.completion_tokens,
                },
            };
            Json(resp).into_response()
        }
        Ok(Err(e)) => ApiError::new(e.to_string(), "inference_error").into_response(),
        Err(e) => ApiError::new(format!("join: {e}"), "internal_error").into_response(),
    }
}

#[allow(dead_code)]
fn _suppress_unused_engine_state() -> EngineState { EngineState::Idle }

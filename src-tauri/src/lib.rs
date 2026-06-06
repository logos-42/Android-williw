//! Williw — Tauri 2 lib entry (used by both desktop `main.rs` and Android mobile entry).
//!
//! v0.1.1 重大改动：
//!   - 引入「算力开关」：默认关闭，点击开启后才对外提供 API。
//!   - 开启时按需绑定端口并生成地址 + 密钥（QR 显示），类似 WiFi 直连。
//!   - 关闭时停止监听并清理。
//!   - 新增 `cmd_download_stream` 真实流式下载（带进度事件）。

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
use tauri::{AppHandle, Emitter, Manager, State as TauriState};
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::runtime::Runtime;
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
    theme: String,
    /// 启动时是否自动开启算力服务（默认 false：手动开关）
    auto_start: bool,
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
            allow_external_access: true, // v0.1.1 默认允许局域网
            theme: "dark".into(),
            auto_start: false,
        }
    }
}

fn settings_path(app: &AppHandle) -> PathBuf {
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

fn generate_api_key() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let mut s = format!("wlw-{:x}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos());
    // 简单补充随机性
    let mut buf = [0u8; 16];
    if let Ok(_) = std::fs::File::open("/dev/urandom").map(|_| ()) {
        // 读取 urandom（部分 Android 设备存在）
        if let Ok(f) = std::fs::File::open("/dev/urandom") {
            use std::io::Read;
            let mut f = f;
            let _ = f.read_exact(&mut buf);
            for b in buf.iter() { s.push_str(&format!("{:02x}", b)); }
        }
    }
    s
}

// ===== HTTP 共享状态 =====

#[derive(Clone)]
struct ApiAppState {
    engine: Arc<Engine>,
    model_list: Vec<ModelInfo>,
    api_key: Option<String>,
    #[allow(dead_code)]
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
struct Choice { index: u32, message: ChatMessage, finish_reason: String }

#[derive(Debug, Serialize)]
struct Usage { prompt_tokens: u32, completion_tokens: u32, total_tokens: u32 }

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
// v0.1.1：动态启停。一个 tokio runtime 长期持有；每次开启新开一个 axum 任务并
// 持有 shutdown 句柄，关闭时发送信号。

struct ApiServerHandle {
    shutdown_tx: Option<oneshot::Sender<()>>,
    bound_port: u16,
    bound_addr: SocketAddr,
    rt: Arc<Runtime>,
}

fn build_router(state: ApiAppState) -> Router {
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any).allow_headers(Any);
    Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/status", get(status_route))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn run_axum_to_shutdown(listener: TcpListener, app: Router, mut shutdown: oneshot::Receiver<()>) {
    let server = axum::serve(listener, app);
    tokio::select! {
        res = server => {
            if let Err(e) = res { tracing::error!("axum server error: {e}"); }
        }
        _ = &mut shutdown => {
            tracing::info!("axum server received shutdown signal");
        }
    }
}

fn start_api_server(
    state: ApiAppState,
    port: u16,
    allow_external: bool,
) -> Result<(u16, SocketAddr, oneshot::Sender<()>, Arc<Runtime>), Box<dyn std::error::Error>> {
    let rt = Arc::new(Runtime::new()?);
    let (tx, rx) = oneshot::channel();
    let app = build_router(state);
    let bind: SocketAddr = if allow_external {
        ([0, 0, 0, 0], port).into()
    } else {
        ([127, 0, 0, 1], port).into()
    };
    let (bound_addr, listener) = rt.block_on(async move {
        let listener = TcpListener::bind(bind).await?;
        let addr = listener.local_addr()?;
        Ok::<_, std::io::Error>((addr, listener))
    })?;
    let port = bound_addr.port();
    tracing::info!("williw-api-server listening on http://{bound_addr}");
    for lan in collect_lan_addrs(port) {
        tracing::info!("  · lan: {lan}");
    }
    let _ = rt.spawn(run_axum_to_shutdown(listener, app, rx));
    Ok((port, bound_addr, tx, rt))
}

// ===== 运行时：动态开关 =====

pub struct AppContext {
    engine: Arc<Engine>,
    settings: Arc<Mutex<AppSettings>>,
    settings_path: PathBuf,
    api_handle: Arc<Mutex<Option<ApiServerHandle>>>,
    api_enabled: Arc<Mutex<bool>>,
}

impl Clone for AppContext {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            settings: self.settings.clone(),
            settings_path: self.settings_path.clone(),
            api_handle: self.api_handle.clone(),
            api_enabled: self.api_enabled.clone(),
        }
    }
}

impl AppContext {
    fn stop_api(&self) {
        let mut h = self.api_handle.lock();
        if let Some(mut srv) = h.take() {
            if let Some(tx) = srv.shutdown_tx.take() {
                let _ = tx.send(());
            }
            // 保留 rt 以便复用：Runtime 在最后一个 task 退出时自动结束
            // 但我们要让现有 server 任务真正退出，所以必须让 rx 收到信号后能结束循环
            // 这里采用「创建新 rt」策略更稳，下方 start_api 每次新建
        }
        *self.api_enabled.lock() = false;
    }

    fn start_api(&self, app: &AppHandle) -> Result<u16, String> {
        let s = self.settings.lock().clone();
        let api_key = if s.api_key.is_none() {
            let k = generate_api_key();
            let mut s_mut = self.settings.lock();
            s_mut.api_key = Some(k.clone());
            let _ = save_settings(&self.settings_path, &s_mut);
            k
        } else {
            s.api_key.clone().unwrap()
        };

        let engine = self.engine.clone();
        let settings_arc = self.settings.clone();
        let api_state = ApiAppState {
            engine: engine.clone(),
            model_list: vec![ModelInfo {
                id: engine.status().model_id.clone().unwrap_or_else(|| "williw-local".into()),
                object: "model".into(),
                created: chrono::Utc::now().timestamp(),
                owned_by: "williw".into(),
            }],
            api_key: Some(api_key),
            settings: settings_arc.clone(),
        };

        // 先停掉旧实例
        self.stop_api();

        let allow_external = s.allow_external_access;
        let port = s.api_port;
        match start_api_server(api_state, port, allow_external) {
            Ok((bound_port, bound_addr, tx, rt)) => {
                *self.api_handle.lock() = Some(ApiServerHandle {
                    shutdown_tx: Some(tx),
                    bound_port,
                    bound_addr,
                    rt,
                });
                *self.api_enabled.lock() = true;

                // 通知前端（含局域网地址）
                let lan = collect_lan_addrs(bound_port);
                let _ = app.emit("api://started", serde_json::json!({
                    "port": bound_port,
                    "addr": bound_addr.to_string(),
                    "lan_addrs": lan,
                }));
                Ok(bound_port)
            }
            Err(e) => Err(format!("failed to start api: {e}")),
        }
    }
}

// ===== Tauri Entry =====

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let _ = tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,williw_core=info,williw_tauri=info,tower_http=info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .try_init();

    tauri::Builder::default()
        .setup(|app| {
            let s_path = settings_path(&app.handle());
            let settings = load_settings(&s_path);
            tracing::info!("settings loaded from {} (api_port={})", s_path.display(), settings.api_port);

            let model_dir = model_dir_for(&settings);
            tracing::info!("model dir: {}", model_dir.display());
            let engine = Engine::new();
            engine.configure(EngineConfig::default_for_dir(model_dir));
            if let Err(e) = engine.load() {
                tracing::warn!("engine load returned: {e} (continuing, see /v1/status)");
            }

            let settings_arc = Arc::new(Mutex::new(settings.clone()));
            let ctx = AppContext {
                engine: engine.clone(),
                settings: settings_arc.clone(),
                settings_path: s_path.clone(),
                api_handle: Arc::new(Mutex::new(None)),
                api_enabled: Arc::new(Mutex::new(false)),
            };
            app.manage(ctx.clone());

            // v0.1.1：默认不自动开启，让用户手动点开关
            // 如果设置中要求 auto_start，则启动一次
            if settings.auto_start {
                if let Err(e) = ctx.start_api(&app.handle()) {
                    tracing::warn!("auto start api failed: {e}");
                }
            }

            if let Some(window) = app.get_webview_window("main") {
                let port = settings.api_port;
                let _ = window.eval(&format!(
                    "window.__WILLIW_API_BASE__ = 'http://127.0.0.1:{port}'; window.__WILLIW_API_PORT__ = {port};"
                ));
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            cmd_app_info, cmd_status, cmd_settings_get, cmd_settings_set,
            cmd_models_list, cmd_unload, cmd_reload, cmd_download,
            // v0.1.1 新增
            cmd_api_set_enabled, cmd_api_status, cmd_settings_set_api_key,
            cmd_download_stream, cmd_app_exit,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// ===== Tauri Commands =====

#[derive(Debug, Serialize)]
struct AppInfo {
    version: String,
    name: String,
    api_port: u16,
    api_base: String,
    platform: &'static str,
    started_at_ms: u64,
    api_key: Option<String>,
}

fn started_at() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}

#[tauri::command]
fn cmd_app_info(state: TauriState<'_, AppContext>) -> AppInfo {
    let s = state.settings.lock();
    let port = s.api_port;
    let key = s.api_key.clone();
    drop(s);
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "Williw".to_string(),
        api_port: port,
        api_base: format!("http://127.0.0.1:{port}"),
        platform: std::env::consts::OS,
        started_at_ms: started_at(),
        api_key: key,
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
    new_settings: AppSettings,
    state: TauriState<'_, AppContext>,
) -> Result<AppSettings, String> {
    *state.settings.lock() = new_settings.clone();
    save_settings(&state.settings_path, &new_settings).map_err(|e| e.to_string())?;
    tracing::info!("settings saved to {}", state.settings_path.display());
    Ok(new_settings)
}

#[tauri::command]
fn cmd_settings_set_api_key(
    api_key: Option<String>,
    state: TauriState<'_, AppContext>,
) -> Result<AppSettings, String> {
    let mut s = state.settings.lock();
    s.api_key = api_key.clone();
    save_settings(&state.settings_path, &s).map_err(|e| e.to_string())?;
    Ok(s.clone())
}

#[derive(Debug, Serialize)]
struct ModelEntry {
    id: String,
    name: String,
    path: Option<String>,
    size_bytes: Option<u64>,
    state: String,
    is_default: bool,
    backend: String,
}

#[tauri::command]
fn cmd_models_list(state: TauriState<'_, AppContext>) -> Vec<ModelEntry> {
    let dir = model_dir_for(&state.settings.lock());
    let st = state.engine.status();
    let backend = std::env::var("WILLIW_BACKEND").unwrap_or_else(|_| "mock".into());
    let default_id = state.settings.lock().default_model.clone();
    let mut out = Vec::new();

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
struct ReloadRequest { model_path: Option<String> }

#[tauri::command]
fn cmd_reload(req: ReloadRequest, state: TauriState<'_, AppContext>) -> Result<CmdStatus, String> {
    let s = state.settings.lock().clone();
    let mut cfg = EngineConfig::default_for_dir(model_dir_for(&s));
    if let Some(p) = req.model_path { cfg.model_path = PathBuf::from(p); }
    state.engine.unload();
    state.engine.configure(cfg);
    state.engine.load().map_err(|e| e.to_string())?;
    Ok(cmd_status(state))
}

#[derive(Debug, Deserialize)]
struct DownloadRequest { url: String, filename: String }

#[derive(Debug, Serialize)]
struct DownloadResponse { ok: bool, message: String, saved_to: Option<String> }

#[tauri::command]
fn cmd_download(req: DownloadRequest, state: TauriState<'_, AppContext>) -> DownloadResponse {
    let dir = model_dir_for(&state.settings.lock());
    if let Err(e) = std::fs::create_dir_all(&dir) {
        return DownloadResponse { ok: false, message: format!("create dir: {e}"), saved_to: None };
    }
    DownloadResponse {
        ok: false,
        message: format!(
            "MVP does not bundle a downloader. Download manually:\n  curl -L '{}' -o '{}'\nThen it will appear in: {}",
            req.url, dir.join(&req.filename).display(), dir.display()
        ),
        saved_to: None,
    }
}

// ===== v0.1.1 新增命令 =====

#[derive(Debug, Serialize)]
struct ApiStatusPayload {
    enabled: bool,
    port: u16,
    addr: Option<String>,
    api_key: Option<String>,
    model: Option<ModelEntry>,
    /// 局域网可用地址（形如 http://192.168.1.5:8081）
    lan_addrs: Vec<String>,
}

#[tauri::command]
fn cmd_api_status(state: TauriState<'_, AppContext>, _app: AppHandle) -> ApiStatusPayload {
    let enabled = *state.api_enabled.lock();
    let s = state.settings.lock();
    let model = {
        let st = state.engine.status();
        if let Some(id) = st.model_id.clone() {
            Some(ModelEntry {
                id: id.clone(),
                name: id.clone(),
                path: st.model_path.as_ref().map(PathBuf::from).map(|p| p.display().to_string()),
                size_bytes: None,
                state: format!("{:?}", st.state).to_lowercase(),
                is_default: true,
                backend: std::env::var("WILLIW_BACKEND").unwrap_or_else(|_| "mock".into()),
            })
        } else { None }
    };
    let addr = state.api_handle.lock().as_ref().map(|h| h.bound_addr.to_string());
    let lan_addrs = if enabled { collect_lan_addrs(s.api_port) } else { Vec::new() };
    ApiStatusPayload {
        enabled,
        port: s.api_port,
        addr,
        api_key: s.api_key.clone(),
        model,
        lan_addrs,
    }
}

/// 收集本机所有非回环 IPv4 地址，组装成「http://ip:port」列表，
/// 方便其它设备扫码 / 手动填入。失败时返回空数组。
fn collect_lan_addrs(port: u16) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();

    // 主路径：通过 UDP "connect" 让系统挑出本机出口 IP（不真发包）
    if let Ok(sock) = std::net::UdpSocket::bind("0.0.0.0:0") {
        if sock.connect("8.8.8.8:80").is_ok() {
            if let Ok(local) = sock.local_addr() {
                let ip = local.ip();
                if !ip.is_loopback() && !ip.is_unspecified() {
                    out.push(format!("http://{ip}:{port}"));
                }
            }
        }
    }

    // Android 没有 if_addrs crate 也能跑；这里只去重，不强行多网卡枚举
    out.sort();
    out.dedup();
    out
}

#[derive(Debug, Deserialize)]
struct ApiSetEnabledRequest { enabled: bool }

#[tauri::command]
fn cmd_api_set_enabled(
    req: ApiSetEnabledRequest,
    state: TauriState<'_, AppContext>,
    app: AppHandle,
) -> Result<ApiStatusPayload, String> {
    if req.enabled {
        // 校验：必须加载了模型
        let st = state.engine.status();
        if st.model_id.is_none() {
            return Err("未加载任何模型，无法开启算力服务".into());
        }
        let port = state.start_api(&app)?;
        let _ = app.emit("api://started", serde_json::json!({ "port": port }));
        Ok(cmd_api_status(state, app))
    } else {
        state.stop_api();
        let _ = app.emit("api://stopped", serde_json::json!({}));
        Ok(ApiStatusPayload {
            enabled: false,
            port: state.settings.lock().api_port,
            addr: None,
            api_key: state.settings.lock().api_key.clone(),
            model: None,
            lan_addrs: Vec::new(),
        })
    }
}

#[tauri::command]
fn cmd_app_exit(app: AppHandle) {
    app.exit(0);
}

// ===== 流式下载（带进度事件） =====
// 简化的同步下载实现：使用 reqwest blocking 因为我们已经在 spawn 中
// 通过 emit 上报进度。

#[derive(Debug, Deserialize, Clone)]
struct DownloadStreamRequest {
    url: String,
    filename: String,
}

#[derive(Debug, Serialize, Clone)]
struct DownloadProgress {
    percent: f32,
    downloaded: u64,
    total: Option<u64>,
    label: String,
}

#[tauri::command]
fn cmd_download_stream(
    req: DownloadStreamRequest,
    state: TauriState<'_, AppContext>,
    app: AppHandle,
) -> Result<String, String> {
    let dir = model_dir_for(&state.settings.lock());
    std::fs::create_dir_all(&dir).map_err(|e| format!("create dir: {e}"))?;
    let target = dir.join(&req.filename);

    let _ = app.emit("download://progress", DownloadProgress {
        percent: 0.0,
        downloaded: 0,
        total: None,
        label: format!("下载 {}", req.filename),
    });

    // 用 reqwest blocking 在 tauri command 的独立线程中跑（避免阻塞）
    let url = req.url.clone();
    let target_clone = target.clone();
    let app_clone = app.clone();
    let filename = req.filename.clone();

    let result: Result<u64, String> = std::thread::spawn(move || -> Result<u64, String> {
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(60 * 30))
            .build()
            .map_err(|e| format!("client: {e}"))?;
        let mut resp = client.get(&url).send().map_err(|e| format!("get: {e}"))?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let total = resp.content_length();
        let mut file = std::fs::File::create(&target_clone).map_err(|e| format!("create file: {e}"))?;
        let mut downloaded: u64 = 0;
        let mut last_emit = std::time::Instant::now();
        let mut buf = [0u8; 32 * 1024];
        use std::io::{Read, Write};
        loop {
            let n = resp.read(&mut buf).map_err(|e| format!("read: {e}"))?;
            if n == 0 { break; }
            file.write_all(&buf[..n]).map_err(|e| format!("write: {e}"))?;
            downloaded += n as u64;
            if last_emit.elapsed().as_millis() > 200 {
                let percent = match total { Some(t) if t > 0 => (downloaded as f32 / t as f32) * 100.0, _ => 0.0 };
                let _ = app_clone.emit("download://progress", DownloadProgress {
                    percent,
                    downloaded,
                    total,
                    label: format!("{} / {}", human_bytes(downloaded), human_bytes_opt(total)),
                });
                last_emit = std::time::Instant::now();
            }
        }
        file.flush().map_err(|e| format!("flush: {e}"))?;
        Ok(downloaded)
    }).join().unwrap_or_else(|_| Err("thread join failed".into()));

    match result {
        Ok(n) => {
            let _ = app.emit("download://done", serde_json::json!({
                "saved_to": target.display().to_string(),
                "bytes": n,
                "filename": filename,
            }));
            Ok(target.display().to_string())
        }
        Err(e) => {
            let _ = app.emit("download://error", serde_json::json!({"message": e.clone()}));
            Err(e)
        }
    }
}

fn human_bytes(n: u64) -> String {
    if n < 1024 { return format!("{} B", n); }
    if n < 1024*1024 { return format!("{:.1} KB", n as f64 / 1024.0); }
    if n < 1024*1024*1024 { return format!("{:.1} MB", n as f64 / 1024.0 / 1024.0); }
    format!("{:.2} GB", n as f64 / 1024.0 / 1024.0 / 1024.0)
}
fn human_bytes_opt(n: Option<u64>) -> String {
    n.map(human_bytes).unwrap_or_else(|| "—".into())
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
    let result = tokio::task::spawn_blocking(move || engine.generate_blocking(gen_req)).await;
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

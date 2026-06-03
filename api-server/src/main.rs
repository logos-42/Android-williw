//! williw-api-server
//!
//! 独立 Axum HTTP 服务，承载本地推理并提供 OpenAI 兼容的
//! `/v1/chat/completions`、`/v1/models`、`/health` 等端点。
//!
//! 启动模型：
//! - 默认：`WILLIW_BACKEND=mock` — 不需要任何模型文件。
//! - 真实：`WILLIW_BACKEND=candle` + `WILLIW_MODEL_DIR=/path/to/models` + 启用 candle feature。
//!
//! 端口：`WILLIW_API_PORT`（默认 8081）

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{
        IntoResponse, Response,
        sse::{Event, KeepAlive, Sse},
    },
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use williw_core::{ChatMessage, Engine, EngineConfig, GenerateRequest};

#[derive(Clone)]
struct AppState {
    engine: Arc<Engine>,
    model_list: Vec<ModelInfo>,
    api_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelInfo {
    id: String,
    object: String,
    created: i64,
    owned_by: String,
}

#[derive(Debug, Serialize)]
struct ModelsResponse {
    object: String,
    data: Vec<ModelInfo>,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionsRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(default)]
    max_tokens: Option<u32>,
    #[serde(default)]
    temperature: Option<f32>,
    #[serde(default)]
    top_p: Option<f32>,
    #[serde(default)]
    stop: Option<Vec<String>>,
    #[serde(default)]
    stream: Option<bool>,
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
struct ApiError {
    error: ApiErrorBody,
}

#[derive(Debug, Serialize)]
struct ApiErrorBody {
    message: String,
    #[serde(rename = "type")]
    kind: String,
    code: Option<String>,
}

impl ApiError {
    fn new(msg: impl Into<String>, kind: &str) -> Self {
        Self {
            error: ApiErrorBody { message: msg.into(), kind: kind.to_string(), code: None },
        }
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,williw_core=debug,williw_api_server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let model_dir: PathBuf = std::env::var("WILLIW_MODEL_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            // 默认当前目录的 models 子目录；Android 端通过环境变量覆盖
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join("models")
        });

    let cfg = EngineConfig::default_for_dir(model_dir.clone());

    let engine = Engine::new();
    engine.configure(cfg);
    match engine.load() {
        Ok(s) => tracing::info!("engine loaded: state={:?} model_id={:?}", s.state, s.model_id),
        Err(e) => {
            tracing::error!("engine load failed: {e}");
            // mock 后端几乎不会失败；如果失败也继续启动——前端能看到 Error 状态
        }
    }

    let status = engine.status();
    let model_list = vec![ModelInfo {
        id: status.model_id.clone().unwrap_or_else(|| "williw-local".into()),
        object: "model".into(),
        created: chrono::Utc::now().timestamp(),
        owned_by: "williw".into(),
    }];

    let api_key = std::env::var("WILLIW_API_KEY").ok();

    let state = AppState { engine, model_list, api_key };
    let app = build_router(state);

    let port: u16 = std::env::var("WILLIW_API_PORT")
        .ok().and_then(|s| s.parse().ok()).unwrap_or(8081);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = TcpListener::bind(addr).await?;
    tracing::info!("williw-api-server listening on http://{addr}");

    axum::serve(listener, app).await?;
    Ok(())
}

fn build_router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/v1/models", get(list_models))
        .route("/v1/chat/completions", post(chat_completions))
        .route("/v1/status", get(status))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

async fn health() -> &'static str { "ok" }

async fn list_models(State(s): State<AppState>) -> Json<ModelsResponse> {
    Json(ModelsResponse {
        object: "list".into(),
        data: s.model_list.clone(),
    })
}

async fn status(State(s): State<AppState>) -> Json<StatusPayload> {
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
    State(s): State<AppState>,
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

    let gen_req = GenerateRequest {
        messages: req.messages.clone(),
        max_tokens: req.max_tokens.unwrap_or(256),
        temperature: req.temperature.unwrap_or(0.7),
        top_p: req.top_p.unwrap_or(0.9),
        stop: req.stop.clone().unwrap_or_default(),
    };

    let stream = req.stream.unwrap_or(false);
    if stream {
        // MVP：流式返回暂以一次性返回代替（带 SSE Content-Type）
        return sse_fallback(s, req, gen_req).await;
    }

    match s.engine.generate(gen_req).await {
        Ok(r) => {
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
        Err(e) => ApiError::new(e.to_string(), "inference_error").into_response(),
    }
}

/// MVP 阶段：流式走一次性响应（前端不会因此报错）。真流式留待后续 SSE chunk 实现。
async fn sse_fallback(
    s: AppState,
    req: ChatCompletionsRequest,
    gen_req: GenerateRequest,
) -> Response {
    let engine = s.engine.clone();
    match engine.generate(gen_req).await {
        Ok(r) => {
            let chunk = serde_json::json!({
                "id": format!("chatcmpl-{}", uuid::Uuid::new_v4()),
                "object": "chat.completion.chunk",
                "created": chrono::Utc::now().timestamp(),
                "model": req.model,
                "choices": [{
                    "index": 0,
                    "delta": { "role": "assistant", "content": r.text },
                    "finish_reason": "stop"
                }]
            });
            let body = format!("data: {}\n\ndata: [DONE]\n\n", chunk);
            (
                StatusCode::OK,
                [("content-type", "text/event-stream"), ("cache-control", "no-cache")],
                body,
            ).into_response()
        }
        Err(e) => ApiError::new(e.to_string(), "inference_error").into_response(),
    }
}

#[allow(dead_code)]
fn _unused_broadcast() -> broadcast::Sender<String> {
    let (tx, _rx) = broadcast::channel(16);
    tx
}
#[allow(dead_code)]
fn _unused_sse() -> Sse<impl futures_core::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let stream = async_stream::stream! {
        yield Ok(Event::default().data("hello"));
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_state() -> AppState {
        let engine = Engine::new();
        let dir = std::env::temp_dir();
        engine.configure(EngineConfig::default_for_dir(dir));
        engine.load().expect("mock engine load");
        AppState {
            engine,
            model_list: vec![ModelInfo {
                id: "williw-local".into(),
                object: "model".into(),
                created: 0,
                owned_by: "williw".into(),
            }],
            api_key: None,
        }
    }

    #[tokio::test]
    async fn health_returns_ok() {
        let r = health().await;
        assert_eq!(r, "ok");
    }

    #[tokio::test]
    async fn list_models_returns_list() {
        use axum::extract::State;
        let s = test_state();
        let Json(resp) = list_models(State(s)).await;
        assert_eq!(resp.object, "list");
        assert_eq!(resp.data.len(), 1);
    }

    #[tokio::test]
    async fn status_reports_ready() {
        use axum::extract::State;
        let s = test_state();
        let Json(p) = status(State(s)).await;
        assert_eq!(p.state, "ready");
    }

    #[tokio::test]
    async fn chat_completions_returns_choice() {
        use axum::extract::State;
        use axum::Json as AxJson;
        let s = test_state();
        let req = ChatCompletionsRequest {
            model: "williw-local".into(),
            messages: vec![ChatMessage { role: "user".into(), content: "ping".into() }],
            max_tokens: Some(8),
            temperature: None,
            top_p: None,
            stop: None,
            stream: Some(false),
        };
        let resp = chat_completions(State(s), axum::http::HeaderMap::new(), AxJson(req)).await;
        assert_eq!(resp.status(), StatusCode::OK);
    }
}

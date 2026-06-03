//! williw-core: 本地模型推理引擎抽象
//!
//! MVP 提供两套后端：
//! - `MockEngine`（默认）：返回确定性占位响应，保证 release 编译通过、API 可演示。
//! - `GgufEngine`（feature = "candle"）：candle 加载 GGUF Qwen2.5-0.5B-Instruct Q4_K_M。
//!
//! ## 并发模型
//! 内部 `Mutex<Option<LoadedState>>` 保护 LoadedState；`generate` 把内部 Arc
//! 复制后立即 drop 锁，所有推理在 spawn_blocking 线程中同步执行。
//! `busy: AtomicBool` 防止并发推理——移动端内存/算力紧张，单请求串行更稳。

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokenizers::Tokenizer;

#[cfg(feature = "candle")]
pub mod gguf_engine;

mod mock_engine;
pub use mock_engine::MockEngine;

/// 推理错误类型
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("model not loaded")]
    NotLoaded,
    #[error("engine already busy")]
    Busy,
    #[error("model file not found: {0}")]
    ModelMissing(PathBuf),
    #[error("tokenizer error: {0}")]
    Tokenizer(String),
    #[error("inference error: {0}")]
    Inference(String),
    #[error("config error: {0}")]
    Config(String),
}

/// 引擎状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EngineState {
    Idle,
    Loading,
    Ready,
    Generating,
    Error,
}

/// 聊天消息（OpenAI 兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// 单次推理请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,
    #[serde(default = "default_temperature")]
    pub temperature: f32,
    #[serde(default = "default_top_p")]
    pub top_p: f32,
    #[serde(default)]
    pub stop: Vec<String>,
}

fn default_max_tokens() -> u32 { 256 }
fn default_temperature() -> f32 { 0.7 }
fn default_top_p() -> f32 { 0.9 }

/// 推理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResponse {
    pub text: String,
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_ms: u64,
}

/// 引擎对外可观察的状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub state: EngineState,
    pub model_id: Option<String>,
    pub model_path: Option<String>,
    pub tokenizer_path: Option<String>,
    pub context_len: u32,
    pub error: Option<String>,
    pub last_prompt_tokens: Option<u32>,
    pub last_completion_tokens: Option<u32>,
    pub last_total_ms: Option<u64>,
}

/// 引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    pub model_path: PathBuf,
    pub tokenizer_path: Option<PathBuf>,
    pub context_len: u32,
}

impl EngineConfig {
    /// MVP 默认：Qwen2.5-0.5B-Instruct Q4_K_M（~350MB）。
    /// Android 端放在 `filesDir/models/`。
    pub fn default_for_dir(dir: PathBuf) -> Self {
        Self {
            model_path: dir.join("qwen2.5-0.5b-instruct-q4_k_m.gguf"),
            tokenizer_path: Some(dir.join("tokenizer.json")),
            context_len: 2048,
        }
    }
}

/// 后端抽象：未来可换 llama.cpp / 远程
pub trait InferenceBackend: Send + Sync {
    fn model_id(&self) -> &str;
    fn generate(&self, prompt: &str, max_tokens: u32, temperature: f32, top_p: f32, stop: &[String]) -> Result<GenerateResponse, EngineError>;
    fn apply_chat_template(&self, messages: &[ChatMessage]) -> Result<String, EngineError>;
}

struct LoadedState {
    backend: Arc<dyn InferenceBackend>,
    tokenizer: Option<Arc<Tokenizer>>,
}

#[derive(Clone)]
struct StatusSnap {
    state: EngineState,
    model_id: Option<String>,
    model_path: Option<String>,
    tokenizer_path: Option<String>,
    context_len: u32,
    error: Option<String>,
    last_prompt_tokens: Option<u32>,
    last_completion_tokens: Option<u32>,
    last_total_ms: Option<u64>,
}

impl Default for StatusSnap {
    fn default() -> Self {
        Self {
            state: EngineState::Idle,
            model_id: None,
            model_path: None,
            tokenizer_path: None,
            context_len: 0,
            error: None,
            last_prompt_tokens: None,
            last_completion_tokens: None,
            last_total_ms: None,
        }
    }
}

/// 引擎主体
pub struct Engine {
    cfg: Mutex<Option<EngineConfig>>,
    inner: Mutex<Option<LoadedState>>,
    busy: AtomicBool,
    status: Mutex<StatusSnap>,
}

impl Engine {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            cfg: Mutex::new(None),
            inner: Mutex::new(None),
            busy: AtomicBool::new(false),
            status: Mutex::new(StatusSnap::default()),
        })
    }

    /// 配置引擎（多次调用会覆盖）
    pub fn configure(&self, cfg: EngineConfig) {
        *self.cfg.lock().unwrap() = Some(cfg);
    }

    /// 加载模型。
    ///
    /// 行为：
    /// - `mock` 后端：永远成功（即便模型文件不存在——MVP 阶段用于演示）。
    /// - `candle` 后端：校验文件存在、tokenizer 可读；失败返回 [`EngineError::ModelMissing`]。
    pub fn load(&self) -> Result<EngineStatus, EngineError> {
        self.set_state(EngineState::Loading, None);
        let cfg = self.cfg.lock().unwrap().clone()
            .ok_or_else(|| EngineError::Config("engine not configured".into()))?;

        let backend_kind = std::env::var("WILLIW_BACKEND").unwrap_or_else(|_| "mock".into());

        let (model_id_str, model_path_str, tok_path_str, context_len) = match backend_kind.as_str() {
            "candle" => {
                #[cfg(feature = "candle")]
                {
                    if !cfg.model_path.exists() {
                        self.set_state(EngineState::Error, Some(format!("model not found: {}", cfg.model_path.display())));
                        return Err(EngineError::ModelMissing(cfg.model_path.clone()));
                    }
                    let tok_path = cfg.tokenizer_path.clone()
                        .ok_or_else(|| EngineError::Config("tokenizer_path missing".into()))?;
                    if !tok_path.exists() {
                        self.set_state(EngineState::Error, Some(format!("tokenizer not found: {}", tok_path.display())));
                        return Err(EngineError::Config(format!("tokenizer not found: {}", tok_path.display())));
                    }
                    let backend = gguf_engine::GgufEngine::load(&cfg)?;
                    let tokenizer = Tokenizer::from_file(&tok_path)
                        .map_err(|e| EngineError::Tokenizer(e.to_string()))?;
                    let model_id = backend.model_id().to_string();
                    let loaded = LoadedState {
                        backend: Arc::new(backend),
                        tokenizer: Some(Arc::new(tokenizer)),
                    };
                    *self.inner.lock().unwrap() = Some(loaded);
                    (model_id, cfg.model_path.display().to_string(), tok_path.display().to_string(), cfg.context_len)
                }
                #[cfg(not(feature = "candle"))]
                {
                    let _ = cfg;
                    return Err(EngineError::Config("candle feature not enabled in this build".into()));
                }
            }
            _ => {
                // mock 后端：忽略文件存在性；提供一个稳定的 model_id
                let model_id = cfg.model_path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("mock-model")
                    .to_string();
                let backend = MockEngine::new(model_id.clone());
                let loaded = LoadedState {
                    backend: Arc::new(backend),
                    tokenizer: None,
                };
                *self.inner.lock().unwrap() = Some(loaded);
                let tok_path = cfg.tokenizer_path.as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default();
                (model_id, cfg.model_path.display().to_string(), tok_path, cfg.context_len)
            }
        };

        {
            let mut s = self.status.lock().unwrap();
            s.state = EngineState::Ready;
            s.model_id = Some(model_id_str);
            s.model_path = Some(model_path_str);
            s.tokenizer_path = Some(tok_path_str);
            s.context_len = context_len;
            s.error = None;
        }
        Ok(self.status())
    }

    pub fn status(&self) -> EngineStatus {
        let s = self.status.lock().unwrap().clone();
        EngineStatus {
            state: s.state,
            model_id: s.model_id,
            model_path: s.model_path,
            tokenizer_path: s.tokenizer_path,
            context_len: s.context_len,
            error: s.error,
            last_prompt_tokens: s.last_prompt_tokens,
            last_completion_tokens: s.last_completion_tokens,
            last_total_ms: s.last_total_ms,
        }
    }

    /// 同步入口：在 spawn_blocking 中调用
    pub fn generate_blocking(&self, req: GenerateRequest) -> Result<GenerateResponse, EngineError> {
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err(EngineError::Busy);
        }
        let result = self.generate_inner(&req);
        self.busy.store(false, Ordering::SeqCst);
        result
    }

    /// async 包装：自动派发到 spawn_blocking
    pub async fn generate(self: &Arc<Self>, req: GenerateRequest) -> Result<GenerateResponse, EngineError> {
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err(EngineError::Busy);
        }
        let me = self.clone();
        let res = tokio::task::spawn_blocking(move || me.generate_inner(&req)).await;
        self.busy.store(false, Ordering::SeqCst);
        match res {
            Ok(r) => r,
            Err(e) => Err(EngineError::Inference(format!("join: {e}"))),
        }
    }

    fn generate_inner(&self, req: &GenerateRequest) -> Result<GenerateResponse, EngineError> {
        self.set_state(EngineState::Generating, None);

        let (backend, tokenizer) = {
            let g = self.inner.lock().unwrap();
            let loaded = g.as_ref().ok_or(EngineError::NotLoaded)?;
            (loaded.backend.clone(), loaded.tokenizer.clone())
        };

        let result = (|| -> Result<GenerateResponse, EngineError> {
            let prompt = backend.apply_chat_template(&req.messages)?;
            let prompt_n = if let Some(tok) = tokenizer.as_ref() {
                tok.encode(prompt.as_str(), true)
                    .map(|e| e.get_ids().len() as u32)
                    .map_err(|e| EngineError::Tokenizer(e.to_string()))?
            } else {
                (prompt.len() / 4) as u32 // 粗略估计
            };

            let start = Instant::now();
            let mut resp = backend.generate(&prompt, req.max_tokens, req.temperature, req.top_p, &req.stop)?;
            let total_ms = start.elapsed().as_millis() as u64;

            let completion_tokens = if let Some(tok) = tokenizer.as_ref() {
                tok.encode(resp.text.as_str(), false)
                    .map(|e| e.get_ids().len() as u32)
                    .map_err(|e| EngineError::Tokenizer(e.to_string()))?
            } else if resp.completion_tokens == 0 {
                (resp.text.split_whitespace().count() as u32).max(1)
            } else {
                resp.completion_tokens
            };

            resp.prompt_tokens = prompt_n;
            resp.completion_tokens = completion_tokens;
            resp.total_ms = total_ms;
            Ok(resp)
        })();

        self.set_state(EngineState::Ready, None);

        if let Ok(ref r) = result {
            let mut s = self.status.lock().unwrap();
            s.last_prompt_tokens = Some(r.prompt_tokens);
            s.last_completion_tokens = Some(r.completion_tokens);
            s.last_total_ms = Some(r.total_ms);
        }
        result
    }

    fn set_state(&self, state: EngineState, error: Option<String>) {
        let mut s = self.status.lock().unwrap();
        s.state = state;
        s.error = error;
    }

    pub fn unload(&self) {
        *self.inner.lock().unwrap() = None;
        let mut s = self.status.lock().unwrap();
        *s = StatusSnap::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn cfg_for(dir: &std::path::Path) -> EngineConfig {
        EngineConfig {
            model_path: dir.join("model.gguf"),
            tokenizer_path: Some(dir.join("tokenizer.json")),
            context_len: 512,
        }
    }

    #[test]
    fn engine_idle_before_load() {
        let e = Engine::new();
        let s = e.status();
        assert_eq!(s.state, EngineState::Idle);
        assert!(s.model_id.is_none());
    }

    #[test]
    fn mock_engine_load_succeeds_without_files() {
        let e = Engine::new();
        e.configure(cfg_for(&PathBuf::from("/nonexistent")));
        let s = e.load().expect("mock load should succeed");
        assert_eq!(s.state, EngineState::Ready);
        assert!(s.model_id.is_some());
    }

    #[test]
    fn mock_engine_generates_response() {
        let e = Engine::new();
        e.configure(cfg_for(&PathBuf::from("/nonexistent")));
        e.load().unwrap();
        let req = GenerateRequest {
            messages: vec![ChatMessage { role: "user".into(), content: "hi".into() }],
            max_tokens: 16,
            temperature: 0.5,
            top_p: 0.9,
            stop: vec![],
        };
        let r = e.generate_blocking(req).expect("generate ok");
        assert!(r.text.contains("hi"));
        assert!(r.completion_tokens > 0);
    }

    #[test]
    fn chat_template_handles_unknown_role_as_user() {
        // mock_engine 模板：未知角色按 user 处理
        let eng = MockEngine::new("t".into());
        let s = eng.apply_chat_template(&[
            ChatMessage { role: "tool".into(), content: "x".into() },
            ChatMessage { role: "user".into(), content: "y".into() },
        ]).unwrap();
        assert!(s.contains("[user]") || s.contains("user"));
    }

    #[test]
    fn engine_unload_returns_to_idle() {
        let e = Engine::new();
        e.configure(cfg_for(&PathBuf::from("/nonexistent")));
        e.load().unwrap();
        assert_eq!(e.status().state, EngineState::Ready);
        e.unload();
        assert_eq!(e.status().state, EngineState::Idle);
    }
}

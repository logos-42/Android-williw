//! Mock 后端
//!
//! 在没有真模型（或 candle 未编译）时提供可演示的推理：把用户的最后一条
//! `user` 消息回显成简短响应，证明整条 OpenAI 兼容链路能跑通。
//!
//! 真实部署时设置环境变量 `WILLIW_BACKEND=candle` 并启用 `candle` feature，
//! 将切换到 [`crate::gguf_engine::GgufEngine`]。

use std::time::Instant;

use crate::{ChatMessage, EngineError, GenerateResponse, InferenceBackend};

pub struct MockEngine {
    model_id: String,
}

impl MockEngine {
    pub fn new(model_id: String) -> Self { Self { model_id } }
}

impl InferenceBackend for MockEngine {
    fn model_id(&self) -> &str { &self.model_id }

    fn apply_chat_template(&self, messages: &[ChatMessage]) -> Result<String, EngineError> {
        let mut s = String::new();
        for m in messages {
            s.push_str(&format!("[{}] {}\n", m.role, m.content));
        }
        Ok(s)
    }

    fn generate(&self, prompt: &str, max_tokens: u32, temperature: f32, top_p: f32, stop: &[String]) -> Result<GenerateResponse, EngineError> {
        let _ = (temperature, top_p, stop);
        let start = Instant::now();

        let last_user = prompt.lines()
            .rev()
            .find(|l| l.starts_with("[user] "))
            .map(|l| l.trim_start_matches("[user] ").to_string())
            .unwrap_or_else(|| "(no user input)".to_string());

        let response = format!(
            "[mock {}] You said: \"{}\". This is a placeholder response from the bundled mock backend. To enable real local inference, set `WILLIW_BACKEND=candle` and rebuild with `--features candle`.",
            self.model_id,
            last_user.chars().take(200).collect::<String>(),
        );

        // 截断到 max_tokens 估算字符数（粗略：1 token ≈ 4 字符）
        let char_limit = (max_tokens as usize).saturating_mul(4);
        let text = if response.chars().count() > char_limit {
            response.chars().take(char_limit).collect()
        } else {
            response
        };

        Ok(GenerateResponse {
            text,
            prompt_tokens: (prompt.len() / 4) as u32,
            completion_tokens: 0, // 由 generate_inner 补算
            total_ms: start.elapsed().as_millis() as u64,
        })
    }
}

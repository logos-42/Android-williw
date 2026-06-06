//! GGUF 引擎实现：candle 0.8 的 quantized llama/qwen2 路径
//!
//! 通过 `Engine::load` 时设置 `WILLIW_BACKEND=candle` 并启用 `candle` feature
//! 来激活本模块。
//!
//! ## 已知约束
//! - 仅 CPU（Android 上 Metal/Vulkan 需要额外初始化）
//! - tokenizer 必须与模型同源（Qwen2.5 配套的 tokenizer.json）
//! - 模型架构必须为 Llama/Qwen2 系（GGUF 元数据含 `llama` 架构）

use std::time::Instant;

use std::sync::Mutex;

use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_qwen2::ModelWeights;
use tokenizers::Tokenizer;

use crate::{ChatMessage, EngineConfig, EngineError, GenerateResponse, InferenceBackend};

pub struct GgufEngine {
    model_id: String,
    // candle 0.8 的 quantized_qwen2::ModelWeights 不实现 Clone，
    // forward 需要 &mut self；这里包一层 Mutex 满足 InferenceBackend::generate(&self) 的接口契约。
    model: Mutex<ModelWeights>,
    tokenizer: Tokenizer,
    device: Device,
    eos_token: u32,
}

impl GgufEngine {
    pub fn load(cfg: &EngineConfig) -> Result<Self, EngineError> {
        let device = Device::Cpu;
        let model_path = cfg.model_path.to_str()
            .ok_or_else(|| EngineError::Config("non-utf8 model path".into()))?;

        let mut file = std::fs::File::open(model_path)
            .map_err(|e| EngineError::Inference(format!("open gguf: {e}")))?;
        let content = candle_core::quantized::gguf_file::Content::read(&mut file)
            .map_err(|e| EngineError::Inference(format!("read gguf: {e}")))?;
        let model = ModelWeights::from_gguf(content, &mut file, &device)
            .map_err(|e| EngineError::Inference(format!("parse gguf: {e}")))?;

        let tok_path = cfg.tokenizer_path.as_ref()
            .ok_or_else(|| EngineError::Config("tokenizer_path missing".into()))?
            .to_str()
            .ok_or_else(|| EngineError::Config("non-utf8 tokenizer path".into()))?;
        let tokenizer = Tokenizer::from_file(tok_path)
            .map_err(|e| EngineError::Tokenizer(e.to_string()))?;

        let model_id = cfg.model_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("gguf")
            .to_string();

        let vocab = tokenizer.get_vocab(true);
        let eos_token = vocab.get("<|im_end|>")
            .or_else(|| vocab.get("<|end_of_text|>"))
            .or_else(|| vocab.get("</s>"))
            .copied()
            .unwrap_or(0);

        Ok(Self { model_id, model: Mutex::new(model), tokenizer, device, eos_token })
    }
}

impl InferenceBackend for GgufEngine {
    fn model_id(&self) -> &str { &self.model_id }

    fn apply_chat_template(&self, messages: &[ChatMessage]) -> Result<String, EngineError> {
        let mut s = String::new();
        for m in messages {
            let role = m.role.to_lowercase();
            match role.as_str() {
                "system" => {
                    s.push_str("<|im_start|>system\n");
                    s.push_str(&m.content);
                    s.push_str("<|im_end|>\n");
                }
                "user" => {
                    s.push_str("<|im_start|>user\n");
                    s.push_str(&m.content);
                    s.push_str("<|im_end|>\n");
                }
                "assistant" => {
                    s.push_str("<|im_start|>assistant\n");
                    s.push_str(&m.content);
                    s.push_str("<|im_end|>\n");
                }
                _ => {
                    s.push_str("<|im_start|>user\n");
                    s.push_str(&m.content);
                    s.push_str("<|im_end|>\n");
                }
            }
        }
        s.push_str("<|im_start|>assistant\n");
        Ok(s)
    }

    fn generate(&self, prompt: &str, max_tokens: u32, temperature: f32, top_p: f32, stop: &[String]) -> Result<GenerateResponse, EngineError> {
        let start = Instant::now();

        let prompt_ids = self.tokenizer.encode(prompt, true)
            .map_err(|e| EngineError::Tokenizer(e.to_string()))?
            .get_ids().to_vec();
        let prompt_n = prompt_ids.len() as u32;

        let mut model = self.model.lock().unwrap();
        // candle-transformers 0.8.4: quantized_qwen2::ModelWeights::forward 内部不持有 KV cache
        // 每次 forward 都重算 prompt 段的 attention（慢但正确）

        let mut logits_processor = LogitsProcessor::new(0x1234, Some(temperature as f64), Some(top_p as f64));

        // 1) 预热：把 prompt 一次性 forward
        let input = Tensor::new(prompt_ids.as_slice(), &self.device)
            .map_err(|e| EngineError::Inference(format!("tensor: {e}")))?
            .unsqueeze(0)
            .map_err(|e| EngineError::Inference(format!("unsqueeze: {e}")))?;
        let logits = model.forward(&input, 0)
            .map_err(|e| EngineError::Inference(format!("forward: {e}")))?;
        let logits = logits.squeeze(0)
            .map_err(|e| EngineError::Inference(format!("squeeze: {e}")))?;
        let mut next_token = logits_processor.sample(&logits)
            .map_err(|e| EngineError::Inference(format!("sample: {e}")))?;

        // 把 token id 累积成列表，最后一次 tokenizer.decode() 让 BPE 字节还原走正确的算法
        // （避免手动还原 Ġ/▁ 字节出错）
        let mut generated_ids: Vec<u32> = vec![next_token];

        // 2) 自回归（每次 forward 都重算整个 prompt 段的 attention）
        let max_iter = max_tokens.saturating_sub(1) as usize;
        for idx in 0..max_iter {
            if next_token == self.eos_token { break; }
            let input = Tensor::new(&[next_token], &self.device)
                .map_err(|e| EngineError::Inference(format!("tensor: {e}")))?
                .unsqueeze(0)
                .map_err(|e| EngineError::Inference(format!("unsqueeze: {e}")))?;
            let logits = model.forward(&input, prompt_n as usize + idx + 1)
                .map_err(|e| EngineError::Inference(format!("forward: {e}")))?;
            let logits = logits.squeeze(0)
                .map_err(|e| EngineError::Inference(format!("squeeze: {e}")))?;
            next_token = logits_processor.sample(&logits)
                .map_err(|e| EngineError::Inference(format!("sample: {e}")))?;
            generated_ids.push(next_token);
            if !stop.is_empty() {
                // 实时增量检查 stop：把当前累积 ids 解码一下看末尾
                if let Ok(partial) = self.tokenizer.decode(&generated_ids, true) {
                    if stop.iter().any(|s| partial.ends_with(s)) { break; }
                }
            }
        }

        let generated = self.tokenizer.decode(&generated_ids, true)
            .map_err(|e| EngineError::Tokenizer(e.to_string()))?;
        let completion_tokens = generated_ids.len() as u32;

        Ok(GenerateResponse {
            text: generated,
            prompt_tokens: prompt_n,
            completion_tokens,
            total_ms: start.elapsed().as_millis() as u64,
        })
    }
}

        Ok(GenerateResponse {
            text: generated,
            prompt_tokens: prompt_n,
            completion_tokens: 0,
            total_ms: start.elapsed().as_millis() as u64,
        })
    }
}

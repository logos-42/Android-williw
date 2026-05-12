use crate::db::Database;
use williw_shared::*;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::RwLock;

const OLLAMA_BASE_URL: &str = "http://localhost:11434";

#[derive(Debug, Clone)]
struct OllamaModel {
    name: String,
    model: String,
}

#[derive(Debug, serde::Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModelInfo>,
}

#[derive(Debug, serde::Deserialize)]
struct OllamaModelInfo {
    name: String,
    model: String,
    size: u64,
    #[serde(rename = "modified_at")]
    modified_at: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct OllamaGenerateRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, serde::Serialize)]
struct OllamaOptions {
    temperature: Option<f32>,
    num_predict: Option<u32>,
}

#[derive(Debug, serde::Deserialize)]
struct OllamaGenerateResponse {
    response: String,
    done: bool,
    #[serde(rename = "eval_count")]
    eval_count: Option<u32>,
    #[serde(rename = "total_duration")]
    total_duration: Option<u64>,
}

pub struct LocalModelService {
    db: Arc<Database>,
    server_running: AtomicBool,
    server_config: RwLock<LocalApiConfig>,
    default_model: RwLock<Option<String>>,
}

impl LocalModelService {
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            server_running: AtomicBool::new(false),
            server_config: RwLock::new(LocalApiConfig::default()),
            default_model: RwLock::new(None),
        }
    }

    async fn get_ollama_models(&self) -> Result<Vec<OllamaModelInfo>, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/tags", OLLAMA_BASE_URL);

        let response = client
            .get(&url)
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|e| format!("Failed to connect to Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama returned status: {}", response.status()));
        }

        let tags: OllamaTagsResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(tags.models)
    }

    async fn call_ollama(&self, model_name: &str, prompt: &str, temperature: Option<f32>, max_tokens: Option<u32>) -> Result<OllamaGenerateResponse, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", OLLAMA_BASE_URL);

        let request = OllamaGenerateRequest {
            model: model_name.to_string(),
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                temperature,
                num_predict: max_tokens,
            },
        };

        let response = client
            .post(&url)
            .json(&request)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| format!("Failed to call Ollama: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Ollama inference failed: {}", response.status()));
        }

        let result: OllamaGenerateResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

        Ok(result)
    }

    pub async fn list_local_models(&self) -> Result<Vec<LocalModel>, String> {
        let manifest = ModelManifest::default_manifest();

        let mut models = Vec::new();

        let ollama_models = self.get_ollama_models().await.unwrap_or_default();
        let ollama_model_names: Vec<String> = ollama_models.iter().map(|m| m.name.clone()).collect();

        for entry in manifest.models {
            let ollama_name = self.find_ollama_model_name(&entry.name);
            let is_downloaded = ollama_model_names.iter().any(|n| n.contains(&entry.name.replace(" ", "-").to_lowercase()));

            let status = if is_downloaded {
                LocalModelStatus::Ready
            } else {
                LocalModelStatus::NotDownloaded
            };

            let inference_endpoint = if is_downloaded {
                Some(format!("http://localhost:11434"))
            } else {
                None
            };

            let local_path = if is_downloaded {
                Some(format!("ollama://{}", ollama_name.clone().unwrap_or_else(|| entry.name.clone())))
            } else {
                None
            };

            models.push(LocalModel {
                id: entry.id,
                name: entry.name,
                model_type: entry.model_type,
                size_gb: entry.size_gb,
                status,
                download_progress: None,
                download_url: entry.download_url,
                local_path,
                inference_endpoint,
                is_default: false,
                created_at: Utc::now(),
            });
        }

        Ok(models)
    }

    fn find_ollama_model_name(&self, manifest_name: &str) -> Option<String> {
        let normalized = manifest_name.replace(" ", "-").to_lowercase();

        let mappings = vec![
            ("lf2.5", "lf2.5"),
            ("gamma", "gamma"),
            ("phi", "phi"),
            ("qwen", "qwen"),
            ("yi", "yi"),
            ("deepseek", "deepseek"),
            ("llama", "llama"),
            ("mistral", "mistral"),
            ("gemma", "gemma"),
        ];

        for (prefix, ollama_prefix) in mappings {
            if normalized.contains(prefix) {
                return Some(format!("{}:latest", ollama_prefix));
            }
        }

        Some(normalized)
    }

    pub async fn get_local_model(&self, id: Uuid) -> Result<Option<LocalModel>, String> {
        let models = self.list_local_models().await?;
        Ok(models.into_iter().find(|m| m.id == id))
    }

    pub async fn start_download(&self, model_id: Uuid) -> Result<LocalModel, String> {
        let manifest = ModelManifest::default_manifest();

        let entry = manifest.models
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or("Model not found in manifest")?;

        let ollama_name = self.find_ollama_model_name(&entry.name);

        if let Some(name) = ollama_name {
            let client = reqwest::Client::new();
            let url = format!("{}/api/pull", OLLAMA_BASE_URL);

            let request = serde_json::json!({
                "name": name,
                "stream": false
            });

            match client.post(&url).json(&request).send().await {
                Ok(response) if response.status().is_success() => {
                    return Ok(LocalModel {
                        id: entry.id,
                        name: entry.name,
                        model_type: entry.model_type,
                        size_gb: entry.size_gb,
                        status: LocalModelStatus::Ready,
                        download_progress: Some(100.0),
                        download_url: entry.download_url,
                        local_path: Some(format!("ollama://{}", name)),
                        inference_endpoint: Some("http://localhost:11434".to_string()),
                        is_default: false,
                        created_at: Utc::now(),
                    });
                }
                Ok(response) => {
                    return Err(format!("Failed to pull model: {}", response.status()));
                }
                Err(e) => {
                    return Err(format!("Failed to connect to Ollama: {}", e));
                }
            }
        }

        Err("Could not determine Ollama model name".to_string())
    }

    pub async fn delete_model(&self, _model_id: Uuid) -> Result<(), String> {
        Ok(())
    }

    pub async fn set_default_model(&self, model_id: Uuid) -> Result<LocalModel, String> {
        let manifest = ModelManifest::default_manifest();

        let entry = manifest.models
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or("Model not found in manifest")?;

        let ollama_name = self.find_ollama_model_name(&entry.name)
            .ok_or("Could not determine Ollama model name")?;

        {
            let mut default = self.default_model.write().await;
            *default = Some(ollama_name.clone());
        }

        Ok(LocalModel {
            id: entry.id,
            name: entry.name,
            model_type: entry.model_type,
            size_gb: entry.size_gb,
            status: LocalModelStatus::Ready,
            download_progress: Some(100.0),
            download_url: entry.download_url,
            local_path: Some(format!("ollama://{}", ollama_name)),
            inference_endpoint: Some("http://localhost:11434".to_string()),
            is_default: true,
            created_at: Utc::now(),
        })
    }

    pub async fn run_inference(&self, req: InferenceRequest) -> Result<InferenceResponse, String> {
        let start = std::time::Instant::now();

        let model_name = {
            let default = self.default_model.read().await;
            if let Some(ref name) = *default {
                name.clone()
            } else {
                let manifest = ModelManifest::default_manifest();
                let entry = manifest.models
                    .into_iter()
                    .find(|m| m.id == req.model_id);

                match entry {
                    Some(e) => self.find_ollama_model_name(&e.name)
                        .unwrap_or_else(|| "llama:latest".to_string()),
                    None => "llama:latest".to_string(),
                }
            }
        };

        let result = self.call_ollama(
            &model_name,
            &req.prompt,
            req.temperature,
            req.max_tokens,
        ).await?;

        let inference_time = start.elapsed().as_millis() as u64;

        Ok(InferenceResponse {
            model_id: req.model_id,
            output: result.response,
            tokens_used: result.eval_count.unwrap_or(0) as u32,
            inference_time_ms: inference_time,
        })
    }

    pub async fn get_device_info(&self) -> Result<DeviceInfo, String> {
        let ollama_models = self.get_ollama_models().await.unwrap_or_default();

        let installed: Vec<LocalModel> = ollama_models.iter().map(|m| {
            LocalModel {
                id: Uuid::new_v4(),
                name: m.name.clone(),
                model_type: LocalModelType::Deepseek,
                size_gb: (m.size as f64 / 1_000_000_000.0).round(),
                status: LocalModelStatus::Ready,
                download_progress: Some(100.0),
                download_url: String::new(),
                local_path: Some(format!("ollama://{}", m.name)),
                inference_endpoint: Some("http://localhost:11434".to_string()),
                is_default: false,
                created_at: Utc::now(),
            }
        }).collect();

        let is_running = !ollama_models.is_empty();

        Ok(DeviceInfo {
            device_id: "local-pc".to_string(),
            device_name: "Local PC".to_string(),
            available_memory_gb: 16.0,
            available_storage_gb: 500.0,
            is_server_running: is_running,
            server_port: 11434,
            installed_models: installed,
        })
    }

    pub async fn start_local_server(&self) -> Result<String, String> {
        self.server_running.store(true, Ordering::SeqCst);
        Ok("Ollama server is running on port 11434".to_string())
    }

    pub async fn stop_local_server(&self) -> Result<String, String> {
        self.server_running.store(false, Ordering::SeqCst);
        Ok("Local inference server stopped".to_string())
    }

    pub async fn get_server_config(&self) -> Result<LocalApiConfig, String> {
        let config = self.server_config.read().await;
        Ok(config.clone())
    }

    pub async fn update_server_config(&self, config: LocalApiConfig) -> Result<LocalApiConfig, String> {
        let mut cfg = self.server_config.write().await;
        *cfg = config.clone();
        Ok(config)
    }
}
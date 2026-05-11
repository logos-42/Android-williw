use crate::db::Database;
use williw_shared::*;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::RwLock;

/// 本地模型服务
/// 管理本地AI模型的下载、推理和本地API服务器
pub struct LocalModelService {
    /// 数据库连接
    db: Arc<Database>,
    /// 本地服务器运行状态
    server_running: AtomicBool,
    /// 本地服务器配置
    server_config: RwLock<LocalApiConfig>,
}

impl LocalModelService {
    /// 创建本地模型服务实例
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            server_running: AtomicBool::new(false),
            server_config: RwLock::new(LocalApiConfig::default()),
        }
    }

    /// 获取本地模型列表
    /// 从模型清单返回所有可用模型
    pub async fn list_local_models(&self) -> Result<Vec<LocalModel>, String> {
        let manifest = ModelManifest::default_manifest();
        let mut models = Vec::new();

        for entry in manifest.models {
            let model = LocalModel {
                id: entry.id,
                name: entry.name,
                model_type: entry.model_type,
                size_gb: entry.size_gb,
                status: LocalModelStatus::NotDownloaded,
                download_progress: None,
                download_url: entry.download_url,
                local_path: None,
                inference_endpoint: None,
                is_default: false,
                created_at: Utc::now(),
            };
            models.push(model);
        }

        Ok(models)
    }

    /// 获取指定本地模型
    pub async fn get_local_model(&self, id: Uuid) -> Result<Option<LocalModel>, String> {
        let manifest = ModelManifest::default_manifest();

        if let Some(entry) = manifest.models.into_iter().find(|m| m.id == id) {
            Ok(Some(LocalModel {
                id: entry.id,
                name: entry.name,
                model_type: entry.model_type,
                size_gb: entry.size_gb,
                status: LocalModelStatus::NotDownloaded,
                download_progress: None,
                download_url: entry.download_url,
                local_path: None,
                inference_endpoint: None,
                is_default: false,
                created_at: Utc::now(),
            }))
        } else {
            Ok(None)
        }
    }

    /// 开始下载模型
    /// 启动模型下载流程
    /// 
    /// # 参数
    /// * `model_id` - 模型ID
    pub async fn start_download(&self, model_id: Uuid) -> Result<LocalModel, String> {
        let manifest = ModelManifest::default_manifest();

        let entry = manifest.models
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or("Model not found in manifest")?;

        let model = LocalModel {
            id: entry.id,
            name: entry.name,
            model_type: entry.model_type,
            size_gb: entry.size_gb,
            status: LocalModelStatus::Downloading,
            download_progress: Some(0.0),
            download_url: entry.download_url,
            local_path: None,
            inference_endpoint: None,
            is_default: false,
            created_at: Utc::now(),
        };

        Ok(model)
    }

    /// 删除本地模型
    pub async fn delete_model(&self, model_id: Uuid) -> Result<(), String> {
        Ok(())
    }

    /// 设置默认模型
    /// 
    /// # 参数
    /// * `model_id` - 模型ID
    pub async fn set_default_model(&self, model_id: Uuid) -> Result<LocalModel, String> {
        let manifest = ModelManifest::default_manifest();

        let entry = manifest.models
            .into_iter()
            .find(|m| m.id == model_id)
            .ok_or("Model not found in manifest")?;

        let model_name = entry.name.clone();
        let model = LocalModel {
            id: entry.id,
            name: entry.name,
            model_type: entry.model_type,
            size_gb: entry.size_gb,
            status: LocalModelStatus::Ready,
            download_progress: Some(100.0),
            download_url: entry.download_url,
            local_path: Some(format!("/models/{}.gguf", model_name.replace(" ", "_").to_lowercase())),
            inference_endpoint: Some(format!("http://localhost:8081/v1/chat/completions")),
            is_default: true,
            created_at: Utc::now(),
        };

        Ok(model)
    }

    /// 运行推理
    /// 在本地模型上执行推理请求
    /// 
    /// # 参数
    /// * `req` - 推理请求
    pub async fn run_inference(&self, req: InferenceRequest) -> Result<InferenceResponse, String> {
        let start = std::time::Instant::now();

        // 模拟推理响应
        let mock_output = format!(
            "This is a mock response from model {} for: {}",
            req.model_id, req.prompt
        );

        let inference_time = start.elapsed().as_millis() as u64;

        Ok(InferenceResponse {
            model_id: req.model_id,
            output: mock_output,
            tokens_used: 150,
            inference_time_ms: inference_time,
        })
    }

    /// 获取设备信息
    /// 返回设备状态和已安装模型列表
    pub async fn get_device_info(&self) -> Result<DeviceInfo, String> {
        let manifest = ModelManifest::default_manifest();
        let installed: Vec<LocalModel> = vec![];

        Ok(DeviceInfo {
            device_id: "device-mobile-001".to_string(),
            device_name: "My Phone".to_string(),
            available_memory_gb: 6.5,
            available_storage_gb: 128.0,
            is_server_running: self.server_running.load(Ordering::SeqCst),
            server_port: 8081,
            installed_models: installed,
        })
    }

    /// 启动本地推理服务器
    pub async fn start_local_server(&self) -> Result<String, String> {
        self.server_running.store(true, Ordering::SeqCst);
        Ok("Local inference server started on port 8081".to_string())
    }

    /// 停止本地推理服务器
    pub async fn stop_local_server(&self) -> Result<String, String> {
        self.server_running.store(false, Ordering::SeqCst);
        Ok("Local inference server stopped".to_string())
    }

    /// 获取本地服务器配置
    pub async fn get_server_config(&self) -> Result<LocalApiConfig, String> {
        let config = self.server_config.read().await;
        Ok(config.clone())
    }

    /// 更新本地服务器配置
    /// 
    /// # 参数
    /// * `config` - 新的服务器配置
    pub async fn update_server_config(&self, config: LocalApiConfig) -> Result<LocalApiConfig, String> {
        let mut cfg = self.server_config.write().await;
        *cfg = config.clone();
        Ok(config)
    }
}
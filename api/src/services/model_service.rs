use crate::db::Database;
use williw_shared::*;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

/// 模型服务
/// 管理AI模型列表和计算请求
pub struct ModelService {
    /// 数据库连接
    db: Arc<Database>,
}

impl ModelService {
    /// 创建模型服务实例
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    /// 获取AI模型列表
    /// 支持多条件过滤
    /// 
    /// # 参数
    /// * `filter` - 模型过滤条件
    pub async fn list_models(&self, filter: ModelFilter) -> Result<Vec<AiModel>, String> {
        self.db.get_all_models(&filter)
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))
    }

    /// 获取指定模型
    /// 
    /// # 参数
    /// * `id` - 模型ID
    pub async fn get_model(&self, id: Uuid) -> Result<Option<AiModel>, String> {
        self.db.get_model_by_id(&id)
            .await
            .map_err(|e| format!("Failed to fetch model: {}", e))
    }

    /// 创建计算请求
    /// 
    /// # 参数
    /// * `user_id` - 用户ID
    /// * `model_id` - 模型ID
    /// * `amount` - 计算资源数量
    pub async fn create_compute_request(
        &self,
        user_id: Uuid,
        model_id: Uuid,
        amount: f64,
    ) -> Result<ComputeRequest, String> {
        // 验证模型存在
        let model = self.db.get_model_by_id(&model_id)
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("Model not found")?;

        // 创建计算请求
        let request = ComputeRequest {
            id: Uuid::new_v4(),
            user_id,
            model_id,
            amount,
            status: ComputeStatus::Pending,
            result: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.db.create_compute_request(&request)
            .await
            .map_err(|e| format!("{}", e))?;

        Ok(request)
    }

    /// 获取计算请求状态
    /// 
    /// # 参数
    /// * `id` - 请求ID
    pub async fn get_compute_status(&self, id: Uuid) -> Result<Option<ComputeRequest>, String> {
        self.db.get_compute_request(&id)
            .await
            .map_err(|e| format!("{}", e))
    }
}
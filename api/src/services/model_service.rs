use crate::db::Database;
use williw_shared::*;
use uuid::Uuid;
use chrono::Utc;
use std::sync::Arc;

pub struct ModelService {
    db: Arc<Database>,
}

impl ModelService {
    pub fn new(db: Arc<Database>) -> Self {
        Self { db }
    }

    pub async fn list_models(&self, filter: ModelFilter) -> Result<Vec<AiModel>, String> {
        self.db.get_all_models(&filter)
            .await
            .map_err(|e| format!("Failed to fetch models: {}", e))
    }

    pub async fn get_model(&self, id: Uuid) -> Result<Option<AiModel>, String> {
        self.db.get_model_by_id(&id)
            .await
            .map_err(|e| format!("Failed to fetch model: {}", e))
    }

    pub async fn create_compute_request(
        &self,
        user_id: Uuid,
        model_id: Uuid,
        amount: f64,
    ) -> Result<ComputeRequest, String> {
        let model = self.db.get_model_by_id(&model_id)
            .await
            .map_err(|e| format!("{}", e))?
            .ok_or("Model not found")?;

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

    pub async fn get_compute_status(&self, id: Uuid) -> Result<Option<ComputeRequest>, String> {
        self.db.get_compute_request(&id)
            .await
            .map_err(|e| format!("{}", e))
    }
}

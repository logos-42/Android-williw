use axum::{
    Router,
    routing::{get, post},
    extract::{State, Query, Json, Extension, Path},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::models::{ComputeRequestCreate, ModelFilterParams};
use crate::services::ModelService;
use williw_shared::{ApiResponse, AiModel, ComputeRequest, ModelFilter};

pub fn routes(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/models", get(list_models))
        .route("/models/:id", get(get_model))
        .route("/request", post(create_compute_request))
        .route("/status/:id", get(get_compute_status))
        .with_state(state)
}

/// 获取AI模型列表处理器
/// 支持按类别、提供商、算力和价格过滤
async fn list_models(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ModelFilterParams>,
) -> Json<ApiResponse<Vec<AiModel>>> {
    let service = ModelService::new(state.db.clone());
    let filter = ModelFilter::from(params);

    match service.list_models(filter).await {
        Ok(models) => Json(ApiResponse::success(models)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 获取指定模型详情处理器
async fn get_model(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<AiModel>> {
    let service = ModelService::new(state.db.clone());

    match service.get_model(id).await {
        Ok(Some(model)) => Json(ApiResponse::success(model)),
        Ok(None) => Json(ApiResponse::error("Model not found")),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 创建计算请求处理器
async fn create_compute_request(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<ComputeRequestCreate>,
) -> Json<ApiResponse<ComputeRequest>> {
    let service = ModelService::new(state.db.clone());

    match service.create_compute_request(user_id, req.model_id, req.amount).await {
        Ok(compute_req) => Json(ApiResponse::success(compute_req)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 获取计算请求状态处理器
async fn get_compute_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<ComputeRequest>> {
    let service = ModelService::new(state.db.clone());

    match service.get_compute_status(id).await {
        Ok(Some(request)) => Json(ApiResponse::success(request)),
        Ok(None) => Json(ApiResponse::error("Request not found")),
        Err(e) => Json(ApiResponse::error(e)),
    }
}
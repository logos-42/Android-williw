use axum::{
    Router,
    routing::{get, post},
    extract::{Query, Json, Extension, Path},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::models::{ComputeRequestCreate, ModelFilterParams};
use crate::services::ModelService;
use williw_shared::{ApiResponse, AiModel, ComputeRequest, ModelFilter};

pub fn routes() -> Router {
    Router::new()
        .route("/models", get(list_models))
        .route("/models/:id", get(get_model))
        .route("/request", post(create_compute_request))
        .route("/status/:id", get(get_compute_status))
}

async fn list_models(
    Extension(state): Extension<Arc<AppState>>,
    Query(params): Query<ModelFilterParams>,
) -> Json<ApiResponse<Vec<AiModel>>> {
    let service = ModelService::new(state.db.clone());
    let filter = ModelFilter::from(params);

    match service.list_models(filter).await {
        Ok(models) => Json(ApiResponse::success(models)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_model(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<AiModel>> {
    let service = ModelService::new(state.db.clone());

    match service.get_model(id).await {
        Ok(Some(model)) => Json(ApiResponse::success(model)),
        Ok(None) => Json(ApiResponse::error("Model not found")),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn create_compute_request(
    Extension(state): Extension<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<ComputeRequestCreate>,
) -> Json<ApiResponse<ComputeRequest>> {
    let service = ModelService::new(state.db.clone());

    match service.create_compute_request(user_id, req.model_id, req.amount).await {
        Ok(compute_req) => Json(ApiResponse::success(compute_req)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_compute_status(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<ComputeRequest>> {
    let service = ModelService::new(state.db.clone());

    match service.get_compute_status(id).await {
        Ok(Some(request)) => Json(ApiResponse::success(request)),
        Ok(None) => Json(ApiResponse::error("Request not found")),
        Err(e) => Json(ApiResponse::error(e)),
    }
}
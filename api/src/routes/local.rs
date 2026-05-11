use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Json, Path, Extension},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::services::LocalModelService;
use williw_shared::{
    ApiResponse, LocalModel, LocalApiConfig, DeviceInfo, InferenceRequest,
    InferenceResponse, ModelManifest, DownloadRequest,
};

pub fn routes() -> Router {
    Router::new()
        .route("/models", get(list_local_models))
        .route("/models/:id", get(get_local_model))
        .route("/models/download", post(download_model))
        .route("/models/:id/delete", delete(delete_model))
        .route("/models/:id/set-default", post(set_default_model))
        .route("/manifest", get(get_model_manifest))
        .route("/inference", post(run_inference))
        .route("/device-info", get(get_device_info))
        .route("/server/start", post(start_local_server))
        .route("/server/stop", post(stop_local_server))
        .route("/server/config", get(get_server_config).post(update_server_config))
}

async fn list_local_models(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<Vec<LocalModel>>> {
    let service = LocalModelService::new(state.db.clone());

    match service.list_local_models().await {
        Ok(models) => Json(ApiResponse::success(models)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_local_model(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<LocalModel>> {
    let service = LocalModelService::new(state.db.clone());

    match service.get_local_model(id).await {
        Ok(Some(model)) => Json(ApiResponse::success(model)),
        Ok(None) => Json(ApiResponse::error("Model not found")),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn download_model(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<DownloadRequest>,
) -> Json<ApiResponse<LocalModel>> {
    let service = LocalModelService::new(state.db.clone());

    match service.start_download(req.model_id).await {
        Ok(model) => Json(ApiResponse::success(model)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn delete_model(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<()>> {
    let service = LocalModelService::new(state.db.clone());

    match service.delete_model(id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn set_default_model(
    Extension(state): Extension<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<LocalModel>> {
    let service = LocalModelService::new(state.db.clone());

    match service.set_default_model(id).await {
        Ok(model) => Json(ApiResponse::success(model)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_model_manifest() -> Json<ApiResponse<ModelManifest>> {
    Json(ApiResponse::success(ModelManifest::default_manifest()))
}

async fn run_inference(
    Extension(state): Extension<Arc<AppState>>,
    Json(req): Json<InferenceRequest>,
) -> Json<ApiResponse<InferenceResponse>> {
    let service = LocalModelService::new(state.db.clone());

    match service.run_inference(req).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_device_info(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<DeviceInfo>> {
    let service = LocalModelService::new(state.db.clone());

    match service.get_device_info().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn start_local_server(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let service = LocalModelService::new(state.db.clone());

    match service.start_local_server().await {
        Ok(msg) => Json(ApiResponse::success(msg)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn stop_local_server(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let service = LocalModelService::new(state.db.clone());

    match service.stop_local_server().await {
        Ok(msg) => Json(ApiResponse::success(msg)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_server_config(
    Extension(state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<LocalApiConfig>> {
    let service = LocalModelService::new(state.db.clone());

    match service.get_server_config().await {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn update_server_config(
    Extension(state): Extension<Arc<AppState>>,
    Json(config): Json<LocalApiConfig>,
) -> Json<ApiResponse<LocalApiConfig>> {
    let service = LocalModelService::new(state.db.clone());

    match service.update_server_config(config).await {
        Ok(config) => Json(ApiResponse::success(config)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}
use axum::{
    Router,
    routing::{get, post},
    extract::{State, Json, Extension},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::{AppState, auth::wallet};
use crate::models::{LoginRequest, LoginResponse, ProfileResponse};
use crate::services::AuthService;
use williw_shared::{ApiResponse, User};

pub fn routes() -> Router {
    Router::new()
        .route("/wallet/login", post(wallet_login))
        .route("/wallet/verify", post(verify_wallet))
        .route("/profile", get(get_profile))
}

async fn wallet_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    if !wallet::is_valid_eth_address(&req.wallet_address) {
        return Json(ApiResponse::error("Invalid wallet address format"));
    }

    let service = AuthService::new(state.db.clone());

    match service.wallet_login(LoginRequest {
        wallet_address: req.wallet_address.to_lowercase(),
        signature: req.signature,
        message: req.message,
    }).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn verify_wallet(
    Json(req): Json<crate::models::VerifyRequest>,
) -> Json<ApiResponse<bool>> {
    let is_valid = wallet::is_valid_eth_address(&req.wallet_address);

    if !is_valid {
        return Json(ApiResponse::error("Invalid wallet address"));
    }

    Json(ApiResponse::success(true))
}

async fn get_profile(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Json<ApiResponse<ProfileResponse>> {
    let service = AuthService::new(state.db.clone());

    match service.get_profile(user_id).await {
        Ok(profile) => Json(ApiResponse::success(profile)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

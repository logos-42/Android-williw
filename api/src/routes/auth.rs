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

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/wallet/login", post(wallet_login))
        .route("/wallet/verify", post(verify_wallet))
        .route("/profile", get(get_profile))
}

/// 钱包登录处理器
/// 验证钱包签名并返回JWT令牌
async fn wallet_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<LoginRequest>,
) -> Json<ApiResponse<LoginResponse>> {
    // 验证钱包地址格式
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

/// 验证钱包签名处理器
/// 验证钱包地址格式是否有效
async fn verify_wallet(
    Json(req): Json<crate::models::VerifyRequest>,
) -> Json<ApiResponse<bool>> {
    let is_valid = wallet::is_valid_eth_address(&req.wallet_address);

    if !is_valid {
        return Json(ApiResponse::error("Invalid wallet address"));
    }

    Json(ApiResponse::success(true))
}

/// 获取用户资料处理器
/// 返回当前登录用户的详细信息
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
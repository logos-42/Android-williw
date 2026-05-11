use axum::{
    Router,
    routing::{get, post},
    extract::{State, Json, Path, Extension},
};
use std::sync::Arc;
use uuid::Uuid;

use crate::AppState;
use crate::services::PaymentService;
use williw_shared::{ApiResponse, Order, PaymentMethod, PaymentRequest, PaymentResponse};

pub fn routes() -> Router {
    Router::new()
        .route("/create", post(create_order))
        .route("/initiate", post(initiate_payment))
        .route("/status/:id", get(get_payment_status))
        .route("/orders", get(get_user_orders))
        .route("/callback/:provider", post(payment_callback))
}

async fn create_order(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
    Json(req): Json<PaymentRequest>,
) -> Json<ApiResponse<Order>> {
    let service = PaymentService::new(state.db.clone());

    match service.create_order(user_id, req.order_id, 1.0, req.method).await {
        Ok(order) => Json(ApiResponse::success(order)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn initiate_payment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PaymentRequest>,
) -> Json<ApiResponse<PaymentResponse>> {
    let service = PaymentService::new(state.db.clone());

    match service.initiate_payment(req.order_id, req.method).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_payment_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Json<ApiResponse<Order>> {
    let service = PaymentService::new(state.db.clone());

    match service.get_order_status(id).await {
        Ok(Some(order)) => Json(ApiResponse::success(order)),
        Ok(None) => Json(ApiResponse::error("Order not found")),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_user_orders(
    State(state): State<Arc<AppState>>,
    Extension(user_id): Extension<Uuid>,
) -> Json<ApiResponse<Vec<Order>>> {
    let service = PaymentService::new(state.db.clone());

    match service.get_user_orders(user_id).await {
        Ok(orders) => Json(ApiResponse::success(orders)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn payment_callback(
    State(state): State<Arc<AppState>>,
    Path(provider): Path<String>,
    Json(payload): Json<serde_json::Value>,
) -> Json<ApiResponse<String>> {
    let service = PaymentService::new(state.db.clone());

    if let Some(order_id) = payload.get("order_id").and_then(|v| v.as_str()) {
        if let Ok(uuid) = Uuid::parse_str(order_id) {
            if let Err(e) = service.confirm_payment(uuid).await {
                return Json(ApiResponse::error(e));
            }
            return Json(ApiResponse::success(format!("Payment confirmed for {}", provider)));
        }
    }

    Json(ApiResponse::error("Invalid callback payload"))
}

use axum::{
    Router,
    routing::{get, post, delete},
    extract::{Json, Path, Extension},
};
use std::sync::Arc;

use crate::AppState;
use crate::services::P2pService;
use williw_shared::{ApiResponse, P2pConfig, P2pConnectionInfo, P2pStatus, P2pTunnelRequest, P2pTunnelResponse};

pub fn routes() -> Router {
    Router::new()
        .route("/status", get(get_p2p_status))
        .route("/config", get(get_p2p_config).post(update_p2p_config))
        .route("/online", post(p2p_go_online))
        .route("/offline", post(p2p_go_offline))
        .route("/connection-info", get(get_connection_info))
        .route("/connect/:peer_id", post(connect_to_peer))
        .route("/share", post(share_connection))
        .route("/tunnel/:tunnel_id", delete(disconnect_tunnel))
        .route("/test", get(test_connection))
}

async fn get_p2p_status(
    Extension(_state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<P2pStatus>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.get_status().await))
}

async fn get_p2p_config(
    Extension(_state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<P2pConfig>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.get_config().await))
}

async fn update_p2p_config(
    Extension(_state): Extension<Arc<AppState>>,
    Json(config): Json<P2pConfig>,
) -> Json<ApiResponse<P2pConfig>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.update_config(config).await))
}

async fn p2p_go_online(
    Extension(_state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<P2pConnectionInfo>> {
    let service = P2pService::new();

    match service.go_online().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn p2p_go_offline(
    Extension(_state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let service = P2pService::new();

    match service.go_offline().await {
        Ok(msg) => Json(ApiResponse::success(msg)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn get_connection_info(
    Extension(_state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<P2pConnectionInfo>> {
    let service = P2pService::new();

    match service.get_connection_info().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn connect_to_peer(
    Extension(_state): Extension<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Json<ApiResponse<P2pTunnelResponse>> {
    let service = P2pService::new();

    match service.connect_to_peer(peer_id).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn share_connection(
    Extension(_state): Extension<Arc<AppState>>,
    Json(req): Json<P2pTunnelRequest>,
) -> Json<ApiResponse<P2pTunnelResponse>> {
    let service = P2pService::new();

    match service.share_connection(&req.host_peer_id).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn disconnect_tunnel(
    Extension(_state): Extension<Arc<AppState>>,
    Path(tunnel_id): Path<String>,
) -> Json<ApiResponse<()>> {
    let service = P2pService::new();

    match service.disconnect_tunnel(&tunnel_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

async fn test_connection(
    Extension(_state): Extension<Arc<AppState>>,
) -> Json<ApiResponse<williw_shared::ConnectionQuality>> {
    let service = P2pService::new();

    match service.test_connection().await {
        Ok(quality) => Json(ApiResponse::success(quality)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

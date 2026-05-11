use axum::{
    Router,
    routing::{get, post, delete},
    extract::{State, Json, Path, Extension},
};
use std::sync::Arc;

use crate::AppState;
use crate::services::P2pService;
use williw_shared::{ApiResponse, P2pConfig, P2pConnectionInfo, P2pStatus, P2pTunnelRequest, P2pTunnelResponse};

pub fn routes(state: Arc<AppState>) -> Router {
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
        .with_state(state)
}

/// 获取P2P状态处理器
async fn get_p2p_status(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<P2pStatus>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.get_status().await))
}

/// 获取P2P配置处理器
async fn get_p2p_config(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<P2pConfig>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.get_config().await))
}

/// 更新P2P配置处理器
async fn update_p2p_config(
    State(_state): State<Arc<AppState>>,
    Json(config): Json<P2pConfig>,
) -> Json<ApiResponse<P2pConfig>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.update_config(config).await))
}

/// 获取P2P配置处理器
async fn get_p2p_config(
    State(_state): State<Arc<AppState>>,
) -> Json<ApiResponse<P2pConfig>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.get_config().await))
}

/// 更新P2P配置处理器
async fn update_p2p_config(
    State(_state): State<Arc<AppState>>,
    Json(config): Json<P2pConfig>,
) -> Json<ApiResponse<P2pConfig>> {
    let service = P2pService::new();
    Json(ApiResponse::success(service.update_config(config).await))
}

/// P2P上线处理器
async fn p2p_go_online(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<P2pConnectionInfo>> {
    let service = P2pService::new();

    match service.go_online().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// P2P下线处理器
async fn p2p_go_offline(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<String>> {
    let service = P2pService::new();

    match service.go_offline().await {
        Ok(msg) => Json(ApiResponse::success(msg)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 获取连接信息处理器
async fn get_connection_info(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<P2pConnectionInfo>> {
    let service = P2pService::new();

    match service.get_connection_info().await {
        Ok(info) => Json(ApiResponse::success(info)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 连接到对等节点处理器
async fn connect_to_peer(
    State(state): State<Arc<AppState>>,
    Path(peer_id): Path<String>,
) -> Json<ApiResponse<P2pTunnelResponse>> {
    let service = P2pService::new();

    match service.connect_to_peer(peer_id).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 共享连接处理器
async fn share_connection(
    State(state): State<Arc<AppState>>,
    Json(req): Json<P2pTunnelRequest>,
) -> Json<ApiResponse<P2pTunnelResponse>> {
    let service = P2pService::new();

    match service.share_connection(&req.host_peer_id).await {
        Ok(response) => Json(ApiResponse::success(response)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 断开隧道处理器
async fn disconnect_tunnel(
    State(state): State<Arc<AppState>>,
    Path(tunnel_id): Path<String>,
) -> Json<ApiResponse<()>> {
    let service = P2pService::new();

    match service.disconnect_tunnel(&tunnel_id).await {
        Ok(()) => Json(ApiResponse::success(())),
        Err(e) => Json(ApiResponse::error(e)),
    }
}

/// 测试连接处理器
async fn test_connection(
    State(state): State<Arc<AppState>>,
) -> Json<ApiResponse<williw_shared::ConnectionQuality>> {
    let service = P2pService::new();

    match service.test_connection().await {
        Ok(quality) => Json(ApiResponse::success(quality)),
        Err(e) => Json(ApiResponse::error(e)),
    }
}
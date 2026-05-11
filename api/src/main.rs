/// Williw API 主入口模块
/// 模块列表：auth、db、models、payments、routes、services、p2p

mod auth;
mod db;
mod models;
mod payments;
mod routes;
mod services;

// P2P模块公开
pub mod p2p;

use axum::Router;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::db::Database;

/// 应用全局状态
/// 包含数据库连接和JWT密钥
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接
    pub db: Arc<Database>,
    /// JWT密钥
    pub jwt_secret: String,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(database: Database, jwt_secret: String) -> Self {
        Self { db: Arc::new(database), jwt_secret }
    }
}

/// 初始化数据库
/// 创建数据库连接、运行迁移、初始化种子数据
async fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    let database = Database::new().await?;
    database.run_migrations().await?;
    database.seed_models().await?;
    Ok(database)
}

/// Williw API 主函数
/// 启动Web服务器并监听8080端口
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let db = init_db().await?;
    let state = AppState::new(db, "your-super-secret-jwt-key-change-in-production".to_string());

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .nest("/api/auth", routes::auth::routes())
        .nest("/api/compute", routes::compute::routes())
        .nest("/api/payment", routes::payment::routes())
        .nest("/api/local", routes::local::routes())
        .nest("/api/p2p", routes::p2p::routes())
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Williw API server running on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}
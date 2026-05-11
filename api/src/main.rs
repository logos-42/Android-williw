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

use axum::{
    Router,
    middleware,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::auth::jwt::AuthLayer;
use crate::db::Database;

/// 应用全局状态
/// 包含数据库连接和JWT密钥
#[derive(Clone)]
pub struct AppState {
    /// 数据库连接
    pub db: Database,
    /// JWT密钥
    pub jwt_secret: String,
}

impl AppState {
    /// 创建新的应用状态
    pub fn new(database: Database, jwt_secret: String) -> Self {
        Self { db: database, jwt_secret }
    }
}

/// 初始化数据库
/// 创建数据库连接、运行迁移、初始化种子数据
async fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    let database = Database::new().await?;
    // 运行数据库迁移创建表
    database.run_migrations().await?;
    // 初始化默认AI模型数据
    database.seed_models().await?;
    Ok(database)
}

/// Williw API 主函数
/// 启动Web服务器并监听8080端口
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志跟踪
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 初始化数据库
    let db = init_db().await?;
    let state = AppState::new(db, "your-super-secret-jwt-key-change-in-production".to_string());

    // 配置CORS允许跨域请求
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // 构建路由
    let app = Router::new()
        // 认证相关路由
        .nest("/api/auth", routes::auth::routes())
        // 计算资源相关路由
        .nest("/api/compute", routes::compute::routes())
        // 支付相关路由
        .nest("/api/payment", routes::payment::routes())
        // 本地模型相关路由
        .nest("/api/local", routes::local::routes())
        // P2P网络相关路由
        .nest("/api/p2p", routes::p2p::routes())
        // 健康检查端点
        .route("/health", axum::routing::get(|| async { "OK" }))
        // 应用中间件
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    // 绑定TCP监听器
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Williw API server running on http://0.0.0.0:8080");

    // 启动HTTP服务器
    axum::serve(listener, app).await?;

    Ok(())
}
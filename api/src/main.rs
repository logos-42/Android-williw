mod auth;
mod db;
mod models;
mod payments;
mod routes;
mod services;

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

#[derive(Clone)]
pub struct AppState {
    pub db: Database,
    pub jwt_secret: String,
}

impl AppState {
    pub fn new(database: Database, jwt_secret: String) -> Self {
        Self { db: database, jwt_secret }
    }
}

async fn init_db() -> Result<Database, Box<dyn std::error::Error>> {
    let database = Database::new().await?;
    database.run_migrations().await?;
    database.seed_models().await?;
    Ok(database)
}

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
        .route("/health", axum::routing::get(|| async { "OK" }))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::new(state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("Williw API server running on http://0.0.0.0:8080");

    axum::serve(listener, app).await?;

    Ok(())
}

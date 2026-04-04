mod db;

use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use tracing_subscriber::EnvFilter;

async fn health() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status": "ok"})))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let pool = db::init_pool()
        .await
        .expect("failed to initialize database");

    tracing::info!("database initialized");

    let frontend_dist = std::env::var("FRONTEND_DIST").unwrap_or_else(|_| "frontend/dist".into());

    let serve_dir =
        ServeDir::new(&frontend_dist).fallback(ServeFile::new(format!("{frontend_dist}/index.html")));

    let app = Router::new()
        .route("/api/health", get(health))
        .with_state(pool)
        .fallback_service(serve_dir);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    tracing::info!("listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("server error");
}

use std::path::PathBuf;

use axum::{routing::get, Json, Router};
use serde::Serialize;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
}

pub fn router(frontend_dist_dir: PathBuf) -> Router {
    let spa_service = ServeDir::new(&frontend_dist_dir)
        .fallback(tower_http::services::ServeFile::new(frontend_dist_dir.join("index.html")));

    Router::new()
        .route("/api/health", get(health))
        .fallback_service(spa_service)
        .layer(TraceLayer::new_for_http())
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "farminsight-backend",
        version: env!("CARGO_PKG_VERSION"),
    })
}

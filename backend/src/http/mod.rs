use std::path::PathBuf;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    database: &'static str,
}

pub fn router(frontend_dist_dir: PathBuf, db: PgPool) -> Router {
    let spa_service = ServeDir::new(&frontend_dist_dir)
        .fallback(tower_http::services::ServeFile::new(frontend_dist_dir.join("index.html")));

    Router::new()
        .route("/api/health", get(health))
        .fallback_service(spa_service)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState { db })
}

async fn health(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_scalar::<_, i64>("SELECT 1")
        .fetch_one(&state.db)
        .await
    {
        Ok(1) => (
            StatusCode::OK,
            Json(HealthResponse {
                status: "ok",
                service: "farminsight-backend",
                version: env!("CARGO_PKG_VERSION"),
                database: "ok",
            }),
        ),
        Ok(value) => {
            tracing::error!(value, "database health check returned an unexpected value");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unavailable",
                    service: "farminsight-backend",
                    version: env!("CARGO_PKG_VERSION"),
                    database: "unexpected_response",
                }),
            )
        }
        Err(error) => {
            tracing::error!(%error, "database health check failed");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(HealthResponse {
                    status: "unavailable",
                    service: "farminsight-backend",
                    version: env!("CARGO_PKG_VERSION"),
                    database: "unavailable",
                }),
            )
        }
    }
}

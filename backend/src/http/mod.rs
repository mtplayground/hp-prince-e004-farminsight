use std::path::PathBuf;

mod auth;

use crate::{auth::AuthService, config::Settings};
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    auth: Option<AuthService>,
    self_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
    service: &'static str,
    version: &'static str,
    database: &'static str,
}

pub fn router(frontend_dist_dir: PathBuf, db: PgPool, settings: &Settings) -> Router {
    let spa_service = ServeDir::new(&frontend_dist_dir)
        .fallback(tower_http::services::ServeFile::new(frontend_dist_dir.join("index.html")));
    let auth = AuthService::from_settings(db.clone(), settings).ok();

    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/login", get(auth::login))
        .route("/api/auth/register", get(auth::register))
        .route("/api/auth/session", get(auth::session))
        .fallback_service(spa_service)
        .layer(TraceLayer::new_for_http())
        .with_state(AppState {
            db,
            auth,
            self_url: settings.self_url.clone(),
        })
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

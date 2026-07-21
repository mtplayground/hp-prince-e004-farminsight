use std::path::PathBuf;

mod auth;
mod datasets;
mod team_invitations;
mod team_members;
pub mod middleware;

use crate::{auth::AuthService, config::Settings, email::EmailService, storage::StorageClient};
use axum::{
    extract::{DefaultBodyLimit, State},
    http::StatusCode,
    middleware as axum_middleware,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use sqlx::PgPool;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[derive(Clone)]
pub struct AppState {
    db: PgPool,
    auth: Option<AuthService>,
    email: Option<EmailService>,
    storage: Option<StorageClient>,
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
    let email = EmailService::from_settings(settings);
    let storage = StorageClient::from_settings(settings);
    let state = AppState {
        db,
        auth,
        email,
        storage,
        self_url: settings.self_url.clone(),
    };

    let protected_api = Router::new()
        .route("/api/auth/session", get(auth::session))
        .route("/api/auth/context", get(auth::context))
        .route(
            "/api/datasets/upload",
            post(datasets::upload).layer(DefaultBodyLimit::max(datasets::MAX_UPLOAD_BYTES)),
        )
        .route(
            "/api/datasets/preview",
            post(datasets::preview).layer(DefaultBodyLimit::max(datasets::MAX_UPLOAD_BYTES)),
        )
        .route("/api/datasets/:dataset_id/schema", get(datasets::schema))
        .route("/api/datasets/:dataset_id/insights", get(datasets::insights))
        .route(
            "/api/teams/:team_id/datasets/:dataset_id",
            get(datasets::team_dataset),
        )
        .route(
            "/api/teams/:team_id/datasets/:dataset_id/schema",
            get(datasets::team_schema),
        )
        .route(
            "/api/teams/:team_id/datasets/:dataset_id/insights",
            get(datasets::team_insights),
        )
        .route(
            "/api/teams/:team_id/invitations",
            post(team_invitations::create),
        )
        .route("/api/teams/:team_id/members", get(team_members::list))
        .route(
            "/api/team-invitations/:token/accept",
            post(team_invitations::accept),
        )
        .route_layer(axum_middleware::from_fn_with_state(
            state.clone(),
            middleware::require_auth,
        ));

    Router::new()
        .route("/api/health", get(health))
        .route("/api/auth/login", get(auth::login))
        .route("/api/auth/register", get(auth::register))
        .merge(protected_api)
        .fallback_service(spa_service)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
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

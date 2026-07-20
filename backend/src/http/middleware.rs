use axum::{
    body::Body,
    extract::State,
    http::{header, HeaderMap, Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

use crate::{auth::AuthError, models::user::UserIdentity};

use super::AppState;

const TEAM_CONTEXT_HEADER: &str = "x-team-id";

#[derive(Debug, Clone, Serialize)]
pub struct CurrentAuthContext {
    pub user: CurrentUser,
    pub team: CurrentTeamContext,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentUser {
    pub identity: UserIdentity,
    pub is_first_seen: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct CurrentTeamContext {
    pub requested_team_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Debug, thiserror::Error)]
enum AuthMiddlewareError {
    #[error("auth service is not configured")]
    AuthNotConfigured,
    #[error("team context header is invalid")]
    InvalidTeamContext,
    #[error(transparent)]
    Auth(#[from] AuthError),
}

pub async fn require_auth(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    match authenticate_request(&state, request.headers()).await {
        Ok(context) => {
            request.extensions_mut().insert(context);
            next.run(request).await
        }
        Err(error) => error.into_response(),
    }
}

async fn authenticate_request(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<CurrentAuthContext, AuthMiddlewareError> {
    let auth = state
        .auth
        .as_ref()
        .ok_or(AuthMiddlewareError::AuthNotConfigured)?;
    let cookie_header = headers
        .get(header::COOKIE)
        .and_then(|value| value.to_str().ok())
        .ok_or(AuthError::MissingSessionCookie)?;
    let claims = auth.verify_session_cookie(cookie_header).await?;
    let user = auth.upsert_user_from_claims(&claims).await?;
    let team = CurrentTeamContext {
        requested_team_id: requested_team_id(headers)?,
    };

    Ok(CurrentAuthContext {
        user: CurrentUser {
            identity: user.identity,
            is_first_seen: user.is_first_seen,
        },
        team,
    })
}

fn requested_team_id(headers: &HeaderMap) -> Result<Option<String>, AuthMiddlewareError> {
    let Some(value) = headers.get(TEAM_CONTEXT_HEADER) else {
        return Ok(None);
    };
    let team_id = value
        .to_str()
        .map(str::trim)
        .map_err(|_| AuthMiddlewareError::InvalidTeamContext)?;

    if team_id.is_empty() {
        return Ok(None);
    }

    let is_valid = team_id.len() <= 128
        && team_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'));
    if !is_valid {
        return Err(AuthMiddlewareError::InvalidTeamContext);
    }

    Ok(Some(team_id.to_owned()))
}

impl IntoResponse for AuthMiddlewareError {
    fn into_response(self) -> Response {
        match self {
            Self::AuthNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "auth_not_configured",
                }),
            )
                .into_response(),
            Self::InvalidTeamContext => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_team_context",
                }),
            )
                .into_response(),
            Self::Auth(error) => auth_error_response(error),
        }
    }
}

fn auth_error_response(error: AuthError) -> Response {
    match error {
        AuthError::MissingSessionCookie => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "missing_session",
            }),
        )
            .into_response(),
        AuthError::JwksFetch(error) => {
            tracing::error!(%error, "failed to fetch auth JWKS");
            (
                StatusCode::BAD_GATEWAY,
                Json(ErrorResponse {
                    error: "auth_jwks_unavailable",
                }),
            )
                .into_response()
        }
        AuthError::Database(error) => {
            tracing::error!(%error, "failed to upsert authenticated user");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "session_persist_failed",
                }),
            )
                .into_response()
        }
        AuthError::MissingConfig => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "auth_not_configured",
            }),
        )
            .into_response(),
        AuthError::MissingKeyId
        | AuthError::SigningKeyNotFound
        | AuthError::UnsupportedAlgorithm
        | AuthError::MissingEmail
        | AuthError::JwksParse(_)
        | AuthError::Token(_) => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "invalid_session",
            }),
        )
            .into_response(),
    }
}

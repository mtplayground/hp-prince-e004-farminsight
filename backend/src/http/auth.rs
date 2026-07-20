use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    Extension,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::auth::AuthError;

use super::{middleware::CurrentAuthContext, AppState};

#[derive(Debug, Deserialize)]
pub(super) struct AuthRedirectQuery {
    return_to: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionResponse {
    user: crate::models::user::UserIdentity,
    is_first_seen: bool,
    message: String,
}

#[derive(Debug, Serialize)]
pub(super) struct AuthContextResponse {
    session: SessionResponse,
    team: super::middleware::CurrentTeamContext,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum AuthHttpError {
    #[error("auth service is not configured")]
    AuthNotConfigured,
    #[error("return_to must be a frontend path outside /api")]
    InvalidReturnTo,
    #[error(transparent)]
    Auth(#[from] AuthError),
}

pub(super) async fn login(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthRedirectQuery>,
) -> Result<Redirect, AuthHttpError> {
    redirect_to_auth(state, &headers, query.return_to.as_deref())
}

pub(super) async fn register(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<AuthRedirectQuery>,
) -> Result<Redirect, AuthHttpError> {
    redirect_to_auth(state, &headers, query.return_to.as_deref())
}

pub(super) async fn session(
    Extension(context): Extension<CurrentAuthContext>,
) -> Json<SessionResponse> {
    Json(session_response(context))
}

pub(super) async fn context(
    Extension(context): Extension<CurrentAuthContext>,
) -> Json<AuthContextResponse> {
    Json(AuthContextResponse {
        session: session_response(context.clone()),
        team: context.team,
    })
}

fn redirect_to_auth(
    state: AppState,
    headers: &HeaderMap,
    return_to: Option<&str>,
) -> Result<Redirect, AuthHttpError> {
    let auth = state.auth.as_ref().ok_or(AuthHttpError::AuthNotConfigured)?;
    let origin = public_origin(&state, headers)?;
    let return_path = validated_return_path(return_to.unwrap_or("/insights"))?;
    let return_to = format!("{origin}{return_path}");
    let login_url = format!(
        "{}/login?app_token={}&return_to={}",
        auth.auth_url().trim_end_matches('/'),
        urlencoding::encode(auth.app_token()),
        urlencoding::encode(return_to.as_str())
    );

    Ok(Redirect::to(login_url.as_str()))
}

fn session_response(context: CurrentAuthContext) -> SessionResponse {
    let message = if context.user.is_first_seen {
        "Registration complete".to_owned()
    } else {
        match context.user.identity.name.as_deref() {
            Some(name) => format!("Welcome back, {name}"),
            None => "Welcome back".to_owned(),
        }
    };

    SessionResponse {
        user: context.user.identity,
        is_first_seen: context.user.is_first_seen,
        message,
    }
}

fn public_origin(state: &AppState, headers: &HeaderMap) -> Result<String, AuthHttpError> {
    if let Some(self_url) = state.self_url.as_deref() {
        return Ok(self_url.trim_end_matches('/').to_owned());
    }

    let host = forwarded_header(headers, "x-forwarded-host")
        .or_else(|| forwarded_header(headers, "host"))
        .ok_or(AuthHttpError::InvalidReturnTo)?;
    let proto = forwarded_header(headers, "x-forwarded-proto").unwrap_or("https");

    Ok(format!("{proto}://{host}"))
}

fn forwarded_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn validated_return_path(path: &str) -> Result<&str, AuthHttpError> {
    if !path.starts_with('/') || path.starts_with("//") || path.starts_with("/api") {
        return Err(AuthHttpError::InvalidReturnTo);
    }

    Ok(path)
}

impl IntoResponse for AuthHttpError {
    fn into_response(self) -> Response {
        match self {
            Self::AuthNotConfigured => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "auth_not_configured",
                }),
            )
                .into_response(),
            Self::InvalidReturnTo => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_return_to",
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

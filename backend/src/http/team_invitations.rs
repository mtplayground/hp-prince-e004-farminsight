use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sqlx::Row;
use uuid::Uuid;

use crate::{
    email::{EmailDelivery, EmailError},
    http::middleware::CurrentAuthContext,
};

use super::AppState;

const INVITATION_TTL_DAYS: i64 = 7;

#[derive(Debug, Deserialize)]
pub(super) struct CreateInvitationRequest {
    email: String,
}

#[derive(Debug, Serialize)]
pub(super) struct CreateInvitationResponse {
    invitation_id: Uuid,
    team_id: Uuid,
    email: String,
    expires_at: DateTime<Utc>,
    email_delivery: EmailDelivery,
}

#[derive(Debug, Serialize)]
pub(super) struct AcceptInvitationResponse {
    team_id: Uuid,
    role: &'static str,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum InvitationError {
    #[error("request body is invalid")]
    InvalidRequest,
    #[error("team was not found")]
    TeamNotFound,
    #[error("current user must be the team owner")]
    OwnerRequired,
    #[error("invitation was not found")]
    InvitationNotFound,
    #[error("invitation is no longer active")]
    InvitationInactive,
    #[error("invitation has expired")]
    InvitationExpired,
    #[error("invitation email does not match the current user")]
    EmailMismatch,
    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
}

#[derive(Debug)]
struct CreatedInvitation {
    id: Uuid,
    team_id: Uuid,
    email: String,
    team_name: String,
    expires_at: DateTime<Utc>,
    token: String,
}

#[derive(Debug)]
struct StoredInvitation {
    id: Uuid,
    team_id: Uuid,
    email: String,
    expires_at: DateTime<Utc>,
    accepted_at: Option<DateTime<Utc>>,
    revoked_at: Option<DateTime<Utc>>,
}

pub(super) async fn create(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    headers: HeaderMap,
    Path(team_id): Path<Uuid>,
    Json(payload): Json<CreateInvitationRequest>,
) -> Result<Json<CreateInvitationResponse>, InvitationError> {
    let email = normalize_email(payload.email.as_str())?;
    ensure_team_owner(&state, team_id, context.user.identity.sub.as_str()).await?;

    let invitation = create_invitation(
        &state,
        team_id,
        email.as_str(),
        context.user.identity.sub.as_str(),
    )
    .await?;
    let invitation_url = invitation_url(&state, &headers, invitation.token.as_str());
    let delivery = send_invitation_email(
        &state,
        invitation.email.as_str(),
        invitation.team_name.as_str(),
        invitation_url.as_str(),
    )
    .await;

    Ok(Json(CreateInvitationResponse {
        invitation_id: invitation.id,
        team_id: invitation.team_id,
        email: invitation.email,
        expires_at: invitation.expires_at,
        email_delivery: delivery,
    }))
}

pub(super) async fn accept(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    Path(token): Path<String>,
) -> Result<Json<AcceptInvitationResponse>, InvitationError> {
    let token_hash = token_hash(token.as_str());
    let invitation = invitation_by_token_hash(&state, token_hash.as_str()).await?;
    let current_email = normalize_email(context.user.identity.email.as_str())?;

    if invitation.accepted_at.is_some() || invitation.revoked_at.is_some() {
        return Err(InvitationError::InvitationInactive);
    }
    if invitation.expires_at <= Utc::now() {
        return Err(InvitationError::InvitationExpired);
    }
    if normalize_email(invitation.email.as_str())? != current_email {
        return Err(InvitationError::EmailMismatch);
    }

    accept_invitation(
        &state,
        invitation.id,
        invitation.team_id,
        context.user.identity.sub.as_str(),
    )
    .await?;

    Ok(Json(AcceptInvitationResponse {
        team_id: invitation.team_id,
        role: "member",
    }))
}

async fn ensure_team_owner(
    state: &AppState,
    team_id: Uuid,
    user_sub: &str,
) -> Result<(), InvitationError> {
    let row = sqlx::query(
        r#"
        SELECT
          EXISTS(SELECT 1 FROM teams WHERE id = $1) AS team_exists,
          EXISTS(
            SELECT 1
            FROM team_memberships
            WHERE team_id = $1 AND user_sub = $2 AND role = 'owner'
          ) AS is_owner
        "#,
    )
    .bind(team_id)
    .bind(user_sub)
    .fetch_one(&state.db)
    .await?;

    let team_exists: bool = row.try_get("team_exists")?;
    if !team_exists {
        return Err(InvitationError::TeamNotFound);
    }

    let is_owner: bool = row.try_get("is_owner")?;
    if !is_owner {
        return Err(InvitationError::OwnerRequired);
    }

    Ok(())
}

async fn create_invitation(
    state: &AppState,
    team_id: Uuid,
    email: &str,
    invited_by_sub: &str,
) -> Result<CreatedInvitation, InvitationError> {
    let token = invitation_token();
    let token_hash = token_hash(token.as_str());
    let expires_at = Utc::now() + Duration::days(INVITATION_TTL_DAYS);
    let row = sqlx::query(
        r#"
        INSERT INTO team_invitations (team_id, email, token_hash, invited_by_sub, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING
          id,
          team_id,
          email,
          expires_at,
          (SELECT name FROM teams WHERE id = team_invitations.team_id) AS team_name
        "#,
    )
    .bind(team_id)
    .bind(email)
    .bind(token_hash)
    .bind(invited_by_sub)
    .bind(expires_at)
    .fetch_one(&state.db)
    .await?;

    Ok(CreatedInvitation {
        id: row.try_get("id")?,
        team_id: row.try_get("team_id")?,
        email: row.try_get("email")?,
        team_name: row.try_get("team_name")?,
        expires_at: row.try_get("expires_at")?,
        token,
    })
}

async fn invitation_by_token_hash(
    state: &AppState,
    token_hash: &str,
) -> Result<StoredInvitation, InvitationError> {
    let Some(row) = sqlx::query(
        r#"
        SELECT id, team_id, email, expires_at, accepted_at, revoked_at
        FROM team_invitations
        WHERE token_hash = $1
        "#,
    )
    .bind(token_hash)
    .fetch_optional(&state.db)
    .await?
    else {
        return Err(InvitationError::InvitationNotFound);
    };

    Ok(StoredInvitation {
        id: row.try_get("id")?,
        team_id: row.try_get("team_id")?,
        email: row.try_get("email")?,
        expires_at: row.try_get("expires_at")?,
        accepted_at: row.try_get("accepted_at")?,
        revoked_at: row.try_get("revoked_at")?,
    })
}

async fn accept_invitation(
    state: &AppState,
    invitation_id: Uuid,
    team_id: Uuid,
    user_sub: &str,
) -> Result<(), InvitationError> {
    let mut transaction = state.db.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO team_memberships (team_id, user_sub, role)
        VALUES ($1, $2, 'member')
        ON CONFLICT (team_id, user_sub) DO UPDATE SET
          role = CASE
            WHEN team_memberships.role = 'owner' THEN team_memberships.role
            ELSE EXCLUDED.role
          END,
          updated_at = NOW()
        "#,
    )
    .bind(team_id)
    .bind(user_sub)
    .execute(&mut *transaction)
    .await?;

    sqlx::query(
        r#"
        UPDATE team_invitations
        SET accepted_at = NOW(), accepted_by_sub = $2
        WHERE id = $1
        "#,
    )
    .bind(invitation_id)
    .bind(user_sub)
    .execute(&mut *transaction)
    .await?;

    transaction.commit().await?;
    Ok(())
}

async fn send_invitation_email(
    state: &AppState,
    to: &str,
    team_name: &str,
    invitation_url: &str,
) -> EmailDelivery {
    let Some(email) = state.email.as_ref() else {
        return EmailDelivery::skipped();
    };

    let escaped_team_name = html_escape(team_name);
    let escaped_url = html_escape(invitation_url);
    let subject = format!("Invitation to join {team_name}");
    let html = format!(
        "<p>You have been invited to join <strong>{escaped_team_name}</strong>.</p>\
         <p><a href=\"{escaped_url}\">Accept the invitation</a></p>"
    );
    let text = format!("You have been invited to join {team_name}.\nAccept: {invitation_url}");

    match email.send(to, subject.as_str(), html.as_str(), text.as_str()).await {
        Ok(message_id) => EmailDelivery::sent(message_id),
        Err(EmailError::RateLimited) => {
            tracing::warn!(to, "email proxy rate limited team invitation");
            EmailDelivery::rate_limited()
        }
        Err(error) => {
            tracing::error!(%error, to, "failed to send team invitation email");
            EmailDelivery::failed()
        }
    }
}

fn invitation_url(state: &AppState, headers: &HeaderMap, token: &str) -> String {
    let origin = public_origin(state, headers);
    format!("{origin}/team/invitations/{token}")
}

fn public_origin(state: &AppState, headers: &HeaderMap) -> String {
    if let Some(self_url) = state.self_url.as_deref() {
        return self_url.trim_end_matches('/').to_owned();
    }

    let host = forwarded_header(headers, "x-forwarded-host")
        .or_else(|| forwarded_header(headers, "host"))
        .unwrap_or("localhost:8080");
    let proto = forwarded_header(headers, "x-forwarded-proto").unwrap_or("https");

    format!("{proto}://{host}")
}

fn forwarded_header<'a>(headers: &'a HeaderMap, name: &str) -> Option<&'a str> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(',').next())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn normalize_email(email: &str) -> Result<String, InvitationError> {
    let email = email.trim().to_ascii_lowercase();
    let Some((local_part, domain)) = email.split_once('@') else {
        return Err(InvitationError::InvalidRequest);
    };

    if local_part.is_empty()
        || domain.is_empty()
        || domain.contains('@')
        || email.len() > 254
        || email.bytes().any(|byte| byte.is_ascii_control())
    {
        return Err(InvitationError::InvalidRequest);
    }

    Ok(email)
}

fn invitation_token() -> String {
    format!("{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn token_hash(token: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    let digest = Sha256::digest(token.as_bytes());
    let mut encoded = String::with_capacity(digest.len() * 2);
    for byte in digest {
        encoded.push(HEX[(byte >> 4) as usize] as char);
        encoded.push(HEX[(byte & 0x0f) as usize] as char);
    }
    encoded
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

impl IntoResponse for InvitationError {
    fn into_response(self) -> Response {
        match self {
            Self::InvalidRequest => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_request",
                }),
            )
                .into_response(),
            Self::TeamNotFound | Self::InvitationNotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "not_found",
                }),
            )
                .into_response(),
            Self::OwnerRequired | Self::EmailMismatch => (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "forbidden",
                }),
            )
                .into_response(),
            Self::InvitationInactive => (
                StatusCode::CONFLICT,
                Json(ErrorResponse {
                    error: "invitation_inactive",
                }),
            )
                .into_response(),
            Self::InvitationExpired => (
                StatusCode::GONE,
                Json(ErrorResponse {
                    error: "invitation_expired",
                }),
            )
                .into_response(),
            Self::Database(error) => {
                tracing::error!(%error, "team invitation database operation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "database_error",
                    }),
                )
                    .into_response()
            }
        }
    }
}

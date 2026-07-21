use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use uuid::Uuid;

use super::{middleware::CurrentAuthContext, AppState};

#[derive(Debug, Serialize)]
pub(super) struct TeamMembersResponse {
    team_id: Uuid,
    members: Vec<TeamMemberResponse>,
}

#[derive(Debug, Serialize)]
pub(super) struct TeamMemberResponse {
    user_sub: String,
    email: String,
    name: Option<String>,
    picture_url: Option<String>,
    role: String,
    joined_at: DateTime<Utc>,
    last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
}

#[derive(Debug, thiserror::Error)]
pub(super) enum TeamMembersError {
    #[error("team was not found")]
    TeamNotFound,
    #[error("current user must be the team owner")]
    OwnerRequired,
    #[error("database operation failed")]
    Database(#[from] sqlx::Error),
}

pub(super) async fn list(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    Path(team_id): Path<Uuid>,
) -> Result<Json<TeamMembersResponse>, TeamMembersError> {
    ensure_team_owner(&state, team_id, context.user.identity.sub.as_str()).await?;
    let members = list_team_members(&state, team_id).await?;

    Ok(Json(TeamMembersResponse { team_id, members }))
}

async fn ensure_team_owner(
    state: &AppState,
    team_id: Uuid,
    user_sub: &str,
) -> Result<(), TeamMembersError> {
    let (team_exists, is_owner) = sqlx::query_as::<_, (bool, bool)>(
        r#"
        SELECT
          EXISTS(SELECT 1 FROM teams WHERE id = $1),
          EXISTS(
            SELECT 1
            FROM team_memberships
            WHERE team_id = $1 AND user_sub = $2 AND role = 'owner'
          )
        "#,
    )
    .bind(team_id)
    .bind(user_sub)
    .fetch_one(&state.db)
    .await?;

    if !team_exists {
        return Err(TeamMembersError::TeamNotFound);
    }
    if !is_owner {
        return Err(TeamMembersError::OwnerRequired);
    }

    Ok(())
}

async fn list_team_members(
    state: &AppState,
    team_id: Uuid,
) -> Result<Vec<TeamMemberResponse>, TeamMembersError> {
    let members = sqlx::query_as::<
        _,
        (
            String,
            String,
            Option<String>,
            Option<String>,
            String,
            DateTime<Utc>,
            DateTime<Utc>,
        ),
    >(
        r#"
        SELECT
          tm.user_sub,
          u.email,
          u.name,
          u.picture_url,
          tm.role::text AS role,
          tm.created_at AS joined_at,
          u.last_seen_at
        FROM team_memberships tm
        INNER JOIN users u ON u.sub = tm.user_sub
        WHERE tm.team_id = $1
        ORDER BY
          CASE WHEN tm.role = 'owner' THEN 0 ELSE 1 END,
          LOWER(u.email)
        "#,
    )
    .bind(team_id)
    .fetch_all(&state.db)
    .await?
    .into_iter()
    .map(
        |(user_sub, email, name, picture_url, role, joined_at, last_seen_at)| TeamMemberResponse {
            user_sub,
            email,
            name,
            picture_url,
            role,
            joined_at,
            last_seen_at,
        },
    )
    .collect();

    Ok(members)
}

impl IntoResponse for TeamMembersError {
    fn into_response(self) -> Response {
        match self {
            Self::TeamNotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse { error: "not_found" }),
            )
                .into_response(),
            Self::OwnerRequired => (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse { error: "forbidden" }),
            )
                .into_response(),
            Self::Database(error) => {
                tracing::error!(%error, "team member database operation failed");
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

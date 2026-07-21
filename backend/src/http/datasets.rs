use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::types::Json as SqlJson;
use uuid::Uuid;

use crate::{
    charts::select_chart_specs,
    csv_parser::{parse_csv_preview, CsvParseError, CsvPreview},
    insights::generate_insights,
    models::dataset::StoredFileReference,
    profiling::{column_stats_payload, detected_schema_payload, profile_columns, ColumnProfile},
    storage::StorageError,
};

use super::{middleware::CurrentAuthContext, AppState};

pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;

const DEFAULT_CSV_CONTENT_TYPE: &str = "text/csv";

#[derive(Debug, Serialize)]
pub(super) struct UploadResponse {
    dataset: DatasetResponse,
}

#[derive(Debug, Serialize)]
pub(super) struct TeamDatasetResponse {
    dataset: DatasetResponse,
}

#[derive(Debug, Serialize)]
pub(super) struct PreviewResponse {
    preview: CsvPreview,
    profiles: Vec<ColumnProfile>,
}

#[derive(Debug, Serialize)]
pub(super) struct SchemaResponse {
    dataset_id: Uuid,
    owner_sub: String,
    team_id: Option<Uuid>,
    original_filename: String,
    row_count: Option<i64>,
    column_count: Option<i32>,
    column_names: Vec<String>,
    detected_schema: Value,
    column_stats: Value,
    cached_insights: Value,
    cached_chart_specs: Value,
    stats: Value,
    uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub(super) struct InsightsResponse {
    dataset_id: Uuid,
    owner_sub: String,
    team_id: Option<Uuid>,
    original_filename: String,
    insights: Value,
    chart_specs: Value,
    stats: Value,
    uploaded_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct DatasetResponse {
    id: Uuid,
    owner_sub: String,
    team_id: Option<Uuid>,
    original_filename: String,
    storage: StoredFileReference,
    row_count: Option<i64>,
    column_count: Option<i32>,
    column_names: Vec<String>,
    detected_schema: Value,
    column_stats: Value,
    cached_insights: Value,
    cached_chart_specs: Value,
    stats: Value,
    uploaded_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug)]
struct UploadedCsv {
    filename: String,
    content_type: String,
    bytes: Vec<u8>,
}

#[derive(Debug, Serialize)]
struct ErrorResponse {
    error: &'static str,
    message: String,
}

#[derive(Debug, thiserror::Error)]
pub enum UploadError {
    #[error("missing CSV file")]
    MissingFile,
    #[error("invalid CSV file")]
    InvalidCsv,
    #[error("uploaded file is too large")]
    UploadTooLarge,
    #[error("CSV file has headers but no data rows")]
    NoDataRows,
    #[error("invalid team id")]
    InvalidTeamId,
    #[error("team not found")]
    TeamNotFound,
    #[error("current user is not a member of the requested team")]
    ForbiddenTeam,
    #[error("dataset not found")]
    DatasetNotFound,
    #[error(transparent)]
    CsvParse(#[from] CsvParseError),
    #[error(transparent)]
    Multipart(#[from] axum::extract::multipart::MultipartError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

pub(super) async fn preview(
    Extension(_context): Extension<CurrentAuthContext>,
    multipart: Multipart,
) -> Result<Json<PreviewResponse>, UploadError> {
    let (csv, _) = read_csv_upload(multipart).await?;

    if csv.bytes.len() > MAX_UPLOAD_BYTES {
        return Err(UploadError::UploadTooLarge);
    }
    validate_csv_upload(&csv)?;

    let preview = parse_csv_preview(&csv.bytes, 25)?;
    let profiles = profile_columns(&preview);

    Ok(Json(PreviewResponse { preview, profiles }))
}

pub(super) async fn schema(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<SchemaResponse>, UploadError> {
    let schema = fetch_dataset_schema(&state, dataset_id, context.user.identity.sub.as_str())
        .await?
        .ok_or(UploadError::DatasetNotFound)?;

    Ok(Json(schema))
}

pub(super) async fn insights(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<InsightsResponse>, UploadError> {
    let insights = fetch_dataset_insights(&state, dataset_id, context.user.identity.sub.as_str())
        .await?
        .ok_or(UploadError::DatasetNotFound)?;

    Ok(Json(insights))
}

pub(super) async fn team_dataset(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    Path((team_id, dataset_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<TeamDatasetResponse>, UploadError> {
    let dataset = fetch_team_dataset(
        &state,
        team_id,
        dataset_id,
        context.user.identity.sub.as_str(),
    )
    .await?
    .ok_or(UploadError::DatasetNotFound)?;

    Ok(Json(TeamDatasetResponse { dataset }))
}

pub(super) async fn team_schema(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    Path((team_id, dataset_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<SchemaResponse>, UploadError> {
    let schema = fetch_team_dataset_schema(
        &state,
        team_id,
        dataset_id,
        context.user.identity.sub.as_str(),
    )
    .await?
    .ok_or(UploadError::DatasetNotFound)?;

    Ok(Json(schema))
}

pub(super) async fn team_insights(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    Path((team_id, dataset_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<InsightsResponse>, UploadError> {
    let insights = fetch_team_dataset_insights(
        &state,
        team_id,
        dataset_id,
        context.user.identity.sub.as_str(),
    )
    .await?
    .ok_or(UploadError::DatasetNotFound)?;

    Ok(Json(insights))
}

pub(super) async fn upload(
    State(state): State<AppState>,
    Extension(context): Extension<CurrentAuthContext>,
    multipart: Multipart,
) -> Result<(StatusCode, Json<UploadResponse>), UploadError> {
    let storage = state
        .storage
        .as_ref()
        .ok_or(StorageError::MissingConfig)?;
    let (csv, form_team_id) = read_csv_upload(multipart).await?;
    let team_id = form_team_id.or(context.team.requested_team_id);

    if csv.bytes.len() > MAX_UPLOAD_BYTES {
        return Err(UploadError::UploadTooLarge);
    }
    validate_csv_upload(&csv)?;

    if let Some(team_id) = team_id {
        ensure_team_membership(&state, team_id, context.user.identity.sub.as_str()).await?;
    }

    let dataset_id = Uuid::new_v4();
    let storage_key = storage.dataset_key(
        context.user.identity.sub.as_str(),
        dataset_id,
        csv.filename.as_str(),
    );
    let byte_size = i64::try_from(csv.bytes.len()).map_err(|_| UploadError::UploadTooLarge)?;
    let parsed = parse_csv_preview(&csv.bytes, 200)?;
    if parsed.row_count == 0 {
        return Err(UploadError::NoDataRows);
    }
    let profiles = profile_columns(&parsed);
    let detected_schema = detected_schema_payload(&profiles);
    let column_stats = column_stats_payload(&profiles);
    let insights = generate_insights(&parsed, &profiles);
    let chart_specs = select_chart_specs(&parsed, &profiles);
    let cached_insights = json!(insights);
    let cached_chart_specs = json!(chart_specs);
    let row_count = i64::try_from(parsed.row_count).ok();
    let column_count = i32::try_from(parsed.column_count).ok();
    let column_names = parsed.columns.clone();
    let parser_warnings = parsed.warnings.clone();
    let preview_row_count = parsed.rows.len();
    let stats = json!({
        "source": "upload",
        "raw_csv": true,
        "parser": "forgiving",
        "parser_warnings": parser_warnings,
        "preview_row_count": preview_row_count,
        "schema_persisted": true,
        "insights_cached": true,
        "chart_specs_cached": true
    });

    storage
        .put_csv(&storage_key, csv.bytes, csv.content_type.as_str())
        .await?;

    let timestamps = insert_dataset(
        &state,
        InsertDataset {
            id: dataset_id,
            owner_sub: context.user.identity.sub.clone(),
            team_id,
            original_filename: csv.filename.clone(),
            storage_bucket: storage.bucket().to_owned(),
            storage_key: storage_key.clone(),
            content_type: csv.content_type.clone(),
            byte_size,
            row_count,
            column_count,
            column_names: column_names.clone(),
            detected_schema: detected_schema.clone(),
            column_stats: column_stats.clone(),
            cached_insights: cached_insights.clone(),
            cached_chart_specs: cached_chart_specs.clone(),
            stats: stats.clone(),
        },
    )
    .await?;

    Ok((
        StatusCode::CREATED,
        Json(UploadResponse {
            dataset: DatasetResponse {
                id: dataset_id,
                owner_sub: context.user.identity.sub,
                team_id,
                original_filename: csv.filename,
                storage: StoredFileReference {
                    bucket: storage.bucket().to_owned(),
                    key: storage_key,
                    content_type: Some(csv.content_type),
                    byte_size,
                },
                row_count,
                column_count,
                column_names,
                detected_schema,
                column_stats,
                cached_insights,
                cached_chart_specs,
                stats,
                uploaded_at: timestamps.uploaded_at,
                created_at: timestamps.created_at,
                updated_at: timestamps.updated_at,
            },
        }),
    ))
}

async fn read_csv_upload(
    mut multipart: Multipart,
) -> Result<(UploadedCsv, Option<Uuid>), UploadError> {
    let mut csv = None;
    let mut team_id = None;

    while let Some(field) = multipart.next_field().await? {
        let name = field.name().map(str::to_owned);

        match name.as_deref() {
            Some("team_id") => {
                let value = field.text().await?;
                let value = value.trim();
                if !value.is_empty() {
                    team_id =
                        Some(Uuid::parse_str(value).map_err(|_| UploadError::InvalidTeamId)?);
                }
            }
            Some("file") => {
                csv = Some(read_file_field(field).await?);
            }
            _ if csv.is_none() && field.file_name().is_some() => {
                csv = Some(read_file_field(field).await?);
            }
            _ => {}
        }
    }

    Ok((csv.ok_or(UploadError::MissingFile)?, team_id))
}

async fn read_file_field(
    field: axum::extract::multipart::Field<'_>,
) -> Result<UploadedCsv, UploadError> {
    let filename = field
        .file_name()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .ok_or(UploadError::InvalidCsv)?;
    let content_type = field
        .content_type()
        .map(str::to_owned)
        .unwrap_or_else(|| DEFAULT_CSV_CONTENT_TYPE.to_owned());
    let bytes = field.bytes().await?.to_vec();

    Ok(UploadedCsv {
        filename,
        content_type,
        bytes,
    })
}

fn validate_csv_upload(csv: &UploadedCsv) -> Result<(), UploadError> {
    if csv.bytes.is_empty() {
        return Err(UploadError::InvalidCsv);
    }

    let filename_is_csv = csv.filename.to_ascii_lowercase().ends_with(".csv");
    let content_type_is_csv = matches!(
        csv.content_type.to_ascii_lowercase().as_str(),
        "text/csv" | "application/csv" | "application/vnd.ms-excel" | "text/plain"
    );

    if filename_is_csv || content_type_is_csv {
        Ok(())
    } else {
        Err(UploadError::InvalidCsv)
    }
}

async fn ensure_team_membership(
    state: &AppState,
    team_id: Uuid,
    user_sub: &str,
) -> Result<(), UploadError> {
    let (team_exists, is_member) = sqlx::query_as::<_, (bool, bool)>(
        r#"
        SELECT
            EXISTS(SELECT 1 FROM teams WHERE id = $1),
            EXISTS(
                SELECT 1
                FROM team_memberships
                WHERE team_id = $1 AND user_sub = $2
            )
        "#,
    )
    .bind(team_id)
    .bind(user_sub)
    .fetch_one(&state.db)
    .await?;

    if !team_exists {
        return Err(UploadError::TeamNotFound);
    }
    if !is_member {
        return Err(UploadError::ForbiddenTeam);
    }

    Ok(())
}

struct InsertDataset {
    id: Uuid,
    owner_sub: String,
    team_id: Option<Uuid>,
    original_filename: String,
    storage_bucket: String,
    storage_key: String,
    content_type: String,
    byte_size: i64,
    row_count: Option<i64>,
    column_count: Option<i32>,
    column_names: Vec<String>,
    detected_schema: Value,
    column_stats: Value,
    cached_insights: Value,
    cached_chart_specs: Value,
    stats: Value,
}

struct DatasetTimestamps {
    uploaded_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, sqlx::FromRow)]
struct DatasetDetailRow {
    id: Uuid,
    owner_sub: String,
    team_id: Option<Uuid>,
    original_filename: String,
    storage_bucket: String,
    storage_key: String,
    content_type: Option<String>,
    byte_size: i64,
    row_count: Option<i64>,
    column_count: Option<i32>,
    column_names: SqlJson<Vec<String>>,
    detected_schema: SqlJson<Value>,
    column_stats: SqlJson<Value>,
    cached_insights: SqlJson<Value>,
    cached_chart_specs: SqlJson<Value>,
    stats: SqlJson<Value>,
    uploaded_at: DateTime<Utc>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

async fn fetch_dataset_schema(
    state: &AppState,
    dataset_id: Uuid,
    user_sub: &str,
) -> Result<Option<SchemaResponse>, UploadError> {
    let schema = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<Uuid>,
            String,
            Option<i64>,
            Option<i32>,
            SqlJson<Vec<String>>,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            DateTime<Utc>,
        ),
    >(
        r#"
        SELECT
            d.id,
            d.owner_sub,
            d.team_id,
            d.original_filename,
            d.row_count,
            d.column_count,
            d.column_names,
            d.detected_schema,
            d.column_stats,
            d.cached_insights,
            d.cached_chart_specs,
            d.stats,
            d.uploaded_at
        FROM datasets d
        WHERE d.id = $1
          AND (
            d.owner_sub = $2
            OR EXISTS (
              SELECT 1
              FROM team_memberships tm
              WHERE tm.team_id = d.team_id
                AND tm.user_sub = $2
            )
          )
        "#,
    )
    .bind(dataset_id)
    .bind(user_sub)
    .fetch_optional(&state.db)
    .await?
    .map(
        |(
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            row_count,
            column_count,
            SqlJson(column_names),
            SqlJson(detected_schema),
            SqlJson(column_stats),
            SqlJson(cached_insights),
            SqlJson(cached_chart_specs),
            SqlJson(stats),
            uploaded_at,
        )| SchemaResponse {
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            row_count,
            column_count,
            column_names,
            detected_schema,
            column_stats,
            cached_insights,
            cached_chart_specs,
            stats,
            uploaded_at,
        },
    );

    Ok(schema)
}

async fn fetch_dataset_insights(
    state: &AppState,
    dataset_id: Uuid,
    user_sub: &str,
) -> Result<Option<InsightsResponse>, UploadError> {
    let insights = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<Uuid>,
            String,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            DateTime<Utc>,
        ),
    >(
        r#"
        SELECT
            d.id,
            d.owner_sub,
            d.team_id,
            d.original_filename,
            d.cached_insights,
            d.cached_chart_specs,
            d.stats,
            d.uploaded_at
        FROM datasets d
        WHERE d.id = $1
          AND (
            d.owner_sub = $2
            OR EXISTS (
              SELECT 1
              FROM team_memberships tm
              WHERE tm.team_id = d.team_id
                AND tm.user_sub = $2
            )
          )
        "#,
    )
    .bind(dataset_id)
    .bind(user_sub)
    .fetch_optional(&state.db)
    .await?
    .map(
        |(
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            SqlJson(insights),
            SqlJson(chart_specs),
            SqlJson(stats),
            uploaded_at,
        )| InsightsResponse {
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            insights,
            chart_specs,
            stats,
            uploaded_at,
        },
    );

    Ok(insights)
}

async fn fetch_team_dataset(
    state: &AppState,
    team_id: Uuid,
    dataset_id: Uuid,
    user_sub: &str,
) -> Result<Option<DatasetResponse>, UploadError> {
    let dataset = sqlx::query_as::<_, DatasetDetailRow>(
        r#"
        SELECT
            d.id,
            d.owner_sub,
            d.team_id,
            d.original_filename,
            d.storage_bucket,
            d.storage_key,
            d.content_type,
            d.byte_size,
            d.row_count,
            d.column_count,
            d.column_names,
            d.detected_schema,
            d.column_stats,
            d.cached_insights,
            d.cached_chart_specs,
            d.stats,
            d.uploaded_at,
            d.created_at,
            d.updated_at
        FROM datasets d
        WHERE d.id = $2
          AND d.team_id = $1
          AND EXISTS (
            SELECT 1
            FROM team_memberships tm
            WHERE tm.team_id = $1
              AND tm.user_sub = $3
          )
        "#,
    )
    .bind(team_id)
    .bind(dataset_id)
    .bind(user_sub)
    .fetch_optional(&state.db)
    .await?
    .map(|row| DatasetResponse {
        id: row.id,
        owner_sub: row.owner_sub,
        team_id: row.team_id,
        original_filename: row.original_filename,
        storage: StoredFileReference {
            bucket: row.storage_bucket,
            key: row.storage_key,
            content_type: row.content_type,
            byte_size: row.byte_size,
        },
        row_count: row.row_count,
        column_count: row.column_count,
        column_names: row.column_names.0,
        detected_schema: row.detected_schema.0,
        column_stats: row.column_stats.0,
        cached_insights: row.cached_insights.0,
        cached_chart_specs: row.cached_chart_specs.0,
        stats: row.stats.0,
        uploaded_at: row.uploaded_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    });

    Ok(dataset)
}

async fn fetch_team_dataset_schema(
    state: &AppState,
    team_id: Uuid,
    dataset_id: Uuid,
    user_sub: &str,
) -> Result<Option<SchemaResponse>, UploadError> {
    let schema = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<Uuid>,
            String,
            Option<i64>,
            Option<i32>,
            SqlJson<Vec<String>>,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            DateTime<Utc>,
        ),
    >(
        r#"
        SELECT
            d.id,
            d.owner_sub,
            d.team_id,
            d.original_filename,
            d.row_count,
            d.column_count,
            d.column_names,
            d.detected_schema,
            d.column_stats,
            d.cached_insights,
            d.cached_chart_specs,
            d.stats,
            d.uploaded_at
        FROM datasets d
        WHERE d.id = $2
          AND d.team_id = $1
          AND EXISTS (
            SELECT 1
            FROM team_memberships tm
            WHERE tm.team_id = $1
              AND tm.user_sub = $3
          )
        "#,
    )
    .bind(team_id)
    .bind(dataset_id)
    .bind(user_sub)
    .fetch_optional(&state.db)
    .await?
    .map(
        |(
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            row_count,
            column_count,
            SqlJson(column_names),
            SqlJson(detected_schema),
            SqlJson(column_stats),
            SqlJson(cached_insights),
            SqlJson(cached_chart_specs),
            SqlJson(stats),
            uploaded_at,
        )| SchemaResponse {
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            row_count,
            column_count,
            column_names,
            detected_schema,
            column_stats,
            cached_insights,
            cached_chart_specs,
            stats,
            uploaded_at,
        },
    );

    Ok(schema)
}

async fn fetch_team_dataset_insights(
    state: &AppState,
    team_id: Uuid,
    dataset_id: Uuid,
    user_sub: &str,
) -> Result<Option<InsightsResponse>, UploadError> {
    let insights = sqlx::query_as::<
        _,
        (
            Uuid,
            String,
            Option<Uuid>,
            String,
            SqlJson<Value>,
            SqlJson<Value>,
            SqlJson<Value>,
            DateTime<Utc>,
        ),
    >(
        r#"
        SELECT
            d.id,
            d.owner_sub,
            d.team_id,
            d.original_filename,
            d.cached_insights,
            d.cached_chart_specs,
            d.stats,
            d.uploaded_at
        FROM datasets d
        WHERE d.id = $2
          AND d.team_id = $1
          AND EXISTS (
            SELECT 1
            FROM team_memberships tm
            WHERE tm.team_id = $1
              AND tm.user_sub = $3
          )
        "#,
    )
    .bind(team_id)
    .bind(dataset_id)
    .bind(user_sub)
    .fetch_optional(&state.db)
    .await?
    .map(
        |(
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            SqlJson(insights),
            SqlJson(chart_specs),
            SqlJson(stats),
            uploaded_at,
        )| InsightsResponse {
            dataset_id,
            owner_sub,
            team_id,
            original_filename,
            insights,
            chart_specs,
            stats,
            uploaded_at,
        },
    );

    Ok(insights)
}

async fn insert_dataset(
    state: &AppState,
    dataset: InsertDataset,
) -> Result<DatasetTimestamps, UploadError> {
    let (uploaded_at, created_at, updated_at) =
        sqlx::query_as::<_, (DateTime<Utc>, DateTime<Utc>, DateTime<Utc>)>(
            r#"
            INSERT INTO datasets (
                id,
                owner_sub,
                team_id,
                original_filename,
                storage_bucket,
                storage_key,
                content_type,
                byte_size,
                row_count,
                column_count,
                column_names,
                detected_schema,
                column_stats,
                cached_insights,
                cached_chart_specs,
                stats
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            RETURNING uploaded_at, created_at, updated_at
            "#,
        )
        .bind(dataset.id)
        .bind(dataset.owner_sub)
        .bind(dataset.team_id)
        .bind(dataset.original_filename)
        .bind(dataset.storage_bucket)
        .bind(dataset.storage_key)
        .bind(Some(dataset.content_type))
        .bind(dataset.byte_size)
        .bind(dataset.row_count)
        .bind(dataset.column_count)
        .bind(SqlJson(dataset.column_names))
        .bind(SqlJson(dataset.detected_schema))
        .bind(SqlJson(dataset.column_stats))
        .bind(SqlJson(dataset.cached_insights))
        .bind(SqlJson(dataset.cached_chart_specs))
        .bind(SqlJson(dataset.stats))
        .fetch_one(&state.db)
        .await?;

    Ok(DatasetTimestamps {
        uploaded_at,
        created_at,
        updated_at,
    })
}

impl IntoResponse for UploadError {
    fn into_response(self) -> Response {
        match self {
            Self::MissingFile => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "missing_file",
                    message: "Choose a CSV file before previewing or uploading.".to_owned(),
                }),
            )
                .into_response(),
            Self::InvalidCsv => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_csv",
                    message: "Use a CSV, text, semicolon, tab, or pipe-delimited file.".to_owned(),
                }),
            )
                .into_response(),
            Self::UploadTooLarge | Self::Storage(StorageError::UploadTooLarge) => (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(ErrorResponse {
                    error: "upload_too_large",
                    message: "The CSV is larger than the 50 MB upload limit.".to_owned(),
                }),
            )
                .into_response(),
            Self::NoDataRows => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "no_data_rows",
                    message: "The CSV has headers but no data rows to analyze.".to_owned(),
                }),
            )
                .into_response(),
            Self::InvalidTeamId => (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "invalid_team_id",
                    message: "Team ID must be a valid UUID.".to_owned(),
                }),
            )
                .into_response(),
            Self::TeamNotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "team_not_found",
                    message: "That team does not exist.".to_owned(),
                }),
            )
                .into_response(),
            Self::ForbiddenTeam => (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "team_forbidden",
                    message: "You must be a team member to use that team dataset.".to_owned(),
                }),
            )
                .into_response(),
            Self::DatasetNotFound => (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: "dataset_not_found",
                    message: "That dataset was not found or is not shared with you.".to_owned(),
                }),
            )
                .into_response(),
            Self::Multipart(error) => {
                tracing::error!(%error, "failed to read multipart upload");
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "invalid_multipart",
                        message: "The upload form could not be read. Choose the file again."
                            .to_owned(),
                    }),
                )
                    .into_response()
            }
            Self::CsvParse(error) => {
                tracing::error!(%error, "failed to parse CSV preview");
                (
                    StatusCode::BAD_REQUEST,
                    Json(ErrorResponse {
                        error: "invalid_csv",
                        message: format!("{error}. Check the delimiter, quoting, and data rows."),
                    }),
                )
                    .into_response()
            }
            Self::Storage(StorageError::MissingConfig) => (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "object_storage_not_configured",
                    message: "Dataset storage is not configured for this deployment.".to_owned(),
                }),
            )
                .into_response(),
            Self::Storage(error) => {
                tracing::error!(%error, "failed to store uploaded dataset");
                (
                    StatusCode::BAD_GATEWAY,
                    Json(ErrorResponse {
                        error: "object_storage_failed",
                        message: "The CSV parsed successfully, but storage failed. Try again shortly."
                            .to_owned(),
                    }),
                )
                    .into_response()
            }
            Self::Database(error) => {
                tracing::error!(%error, "failed to create dataset metadata record");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: "dataset_create_failed",
                        message: "The CSV parsed successfully, but dataset metadata could not be saved."
                            .to_owned(),
                    }),
                )
                    .into_response()
            }
        }
    }
}

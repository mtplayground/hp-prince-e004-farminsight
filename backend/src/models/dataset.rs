#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dataset {
    pub id: Uuid,
    pub owner_sub: String,
    pub team_id: Option<Uuid>,
    pub original_filename: String,
    pub storage: StoredFileReference,
    pub row_count: Option<i64>,
    pub column_count: Option<i32>,
    pub column_names: Vec<String>,
    pub detected_schema: Value,
    pub column_stats: Value,
    pub cached_insights: Value,
    pub cached_chart_specs: Value,
    pub stats: Value,
    pub uploaded_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredFileReference {
    pub bucket: String,
    pub key: String,
    pub content_type: Option<String>,
    pub byte_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NewDataset {
    pub owner_sub: String,
    pub team_id: Option<Uuid>,
    pub original_filename: String,
    pub storage: StoredFileReference,
    pub row_count: Option<i64>,
    pub column_count: Option<i32>,
    pub column_names: Vec<String>,
    pub detected_schema: Value,
    pub column_stats: Value,
    pub cached_insights: Value,
    pub cached_chart_specs: Value,
    pub stats: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetOwner {
    pub owner_sub: String,
    pub team_id: Option<Uuid>,
}

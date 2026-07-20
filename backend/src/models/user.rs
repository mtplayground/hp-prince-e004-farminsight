#![allow(dead_code)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    pub picture_url: Option<String>,
    pub credentials_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_seen_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserIdentity {
    pub sub: String,
    pub email: String,
    pub name: Option<String>,
    pub picture_url: Option<String>,
}

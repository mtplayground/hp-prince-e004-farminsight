#![allow(dead_code)]

use std::{fmt, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Team {
    pub id: Uuid,
    pub name: String,
    pub created_by_sub: String,
    pub shared_dataset_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamMembership {
    pub team_id: Uuid,
    pub user_sub: String,
    pub role: MembershipRole,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TeamWithMembership {
    pub team: Team,
    pub membership: TeamMembership,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NewTeam {
    pub name: String,
    pub created_by_sub: String,
    pub shared_dataset_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "team_membership_role", rename_all = "lowercase")]
pub enum MembershipRole {
    Owner,
    Member,
}

impl MembershipRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Owner => "owner",
            Self::Member => "member",
        }
    }

    pub fn is_owner(self) -> bool {
        matches!(self, Self::Owner)
    }
}

impl fmt::Display for MembershipRole {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for MembershipRole {
    type Err = MembershipRoleParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "owner" => Ok(Self::Owner),
            "member" => Ok(Self::Member),
            _ => Err(MembershipRoleParseError),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("membership role must be owner or member")]
pub struct MembershipRoleParseError;

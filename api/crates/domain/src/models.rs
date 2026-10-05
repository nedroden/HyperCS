//! Core domain models shared between the db and api crates. No I/O here.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Admin,
    Moderator,
    Member,
}

impl Role {
    pub fn as_db_str(self) -> &'static str {
        match self {
            Role::Admin => "admin",
            Role::Moderator => "moderator",
            Role::Member => "member",
        }
    }

    pub fn from_db_str(s: &str) -> Option<Self> {
        match s {
            "admin" => Some(Role::Admin),
            "moderator" => Some(Role::Moderator),
            "member" => Some(Role::Member),
            _ => None,
        }
    }

    pub fn is_staff(self) -> bool {
        matches!(self, Role::Admin | Role::Moderator)
    }
}

/// The authenticated principal as seen by the policy module. `None` means a guest.
#[derive(Debug, Clone, Copy)]
pub struct AuthUser {
    pub id: Uuid,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastPostSummary {
    pub post_id: Uuid,
    pub topic_id: Uuid,
    pub topic_title: String,
    pub author: UserSummary,
    pub created_at: DateTime<Utc>,
}

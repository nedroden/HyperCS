//! Core domain models shared between the db and api crates. No I/O here.

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// Account role. Serialized lowercase: `admin`, `moderator`, `member`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Admin,
    Moderator,
    Member,
}

impl Role {
    pub const ALL: [Role; 3] = [Role::Admin, Role::Moderator, Role::Member];

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

    /// Decodes a role read from the database. The column has a CHECK
    /// constraint, so an unknown value cannot occur; fall back to the least
    /// privileged role rather than failing a read.
    pub fn from_db_or_member(s: &str) -> Self {
        Self::from_db_str(s).unwrap_or(Role::Member)
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

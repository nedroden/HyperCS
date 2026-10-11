use chrono::{DateTime, Utc};
use hypercs_domain::Role;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::common::page_dto;
use hypercs_domain::pagination::PageParams;

/// What other members may see of an account.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PublicProfile {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub location: Option<String>,
    pub website: Option<String>,
    pub about_me: Option<String>,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

/// A row of the admin user list.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AdminUser {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub role: Role,
    pub post_count: i32,
    pub created_at: DateTime<Utc>,
}

page_dto!(
    /// A page of the admin user list.
    AdminUserPage,
    AdminUser
);

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateRoleRequest {
    pub role: Role,
}

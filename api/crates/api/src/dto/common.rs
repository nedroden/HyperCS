use hypercs_db::users::UserSummaryRow;
use hypercs_domain::Role;
use serde::{Deserialize, Deserializer, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;
use validator::ValidationError;

/// Body of a `201 Created` response; the same URL is in the `Location` header.
#[derive(Debug, Serialize, ToSchema)]
pub struct CreatedResponse {
    pub id: Uuid,
}

/// Pagination query parameters shared by every list endpoint.
#[derive(Debug, Deserialize, IntoParams)]
#[serde(rename_all = "camelCase")]
#[into_params(parameter_in = Query)]
pub struct PageQuery {
    /// 1-based page number (default 1).
    #[param(minimum = 1, example = 1)]
    pub page: Option<u32>,
    /// Items per page. Clamped to 1-100; each endpoint has its own default
    /// (topics 20, posts 15, news 10, users 20, revisions 20).
    #[param(minimum = 1, maximum = 100, example = 20)]
    pub per_page: Option<u32>,
}

/// The author/user summary embedded in other resources.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UserSummary {
    pub id: Uuid,
    pub username: String,
    /// What the UI shows everywhere; defaults to the username.
    pub display_name: String,
    pub role: Role,
}

impl UserSummary {
    pub fn new(id: Uuid, username: String, display_name: String, role: &str) -> Self {
        Self {
            id,
            username,
            display_name,
            role: Role::from_db_or_member(role),
        }
    }

    /// Builds a summary from the `Option` columns of a LEFT JOINed user.
    pub fn from_joined(
        id: Option<Uuid>,
        username: Option<String>,
        display_name: Option<String>,
        role: Option<String>,
    ) -> Option<Self> {
        Some(Self::new(id?, username?, display_name?, &role?))
    }
}

impl From<UserSummaryRow> for UserSummary {
    fn from(row: UserSummaryRow) -> Self {
        Self::new(row.id, row.username, row.display_name, &row.role)
    }
}

/// Defines a concrete, named page schema `{ items, page, perPage, total,
/// totalPages }` for one item type (concrete schemas keep the generated
/// client simple and fully typed).
macro_rules! page_dto {
    ($(#[$meta:meta])* $name:ident, $item:ty) => {
        $(#[$meta])*
        #[derive(Debug, Serialize, ToSchema)]
        #[serde(rename_all = "camelCase")]
        pub struct $name {
            pub items: Vec<$item>,
            /// 1-based page number.
            pub page: u32,
            /// Page size actually applied.
            pub per_page: u32,
            /// Total number of items over all pages.
            pub total: i64,
            pub total_pages: u32,
        }

        impl $name {
            pub fn new(items: Vec<$item>, params: PageParams, total: i64) -> Self {
                Self {
                    items,
                    page: params.page,
                    per_page: params.per_page,
                    total,
                    total_pages: params.total_pages(total),
                }
            }
        }
    };
}
pub(crate) use page_dto;

/// Deserializer for PATCH fields that distinguish "absent" (`None`, leave
/// unchanged) from `null` (`Some(None)`, clear) from a value (`Some(Some(v))`).
pub fn patch_nullable<'de, T, D>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

/// `validator` helper for a nullable PATCH string (`validator` unwraps the
/// `Option<Option<_>>` layers and hands over the string when present).
pub fn max_chars(value: &str, max: u64) -> Result<(), ValidationError> {
    if value.chars().count() as u64 > max {
        let mut error = ValidationError::new("length");
        error.add_param("max".into(), &max);
        Err(error)
    } else {
        Ok(())
    }
}

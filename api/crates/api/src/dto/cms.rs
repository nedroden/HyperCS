use chrono::{DateTime, Utc};
use hypercs_domain::{constants::*, pagination::PageParams};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::common::{UserSummary, page_dto};

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HomepageFeature {
    #[validate(length(min = 1, max = "FEATURE_TITLE_MAX"))]
    #[schema(min_length = 1, max_length = 60)]
    pub title: String,
    #[validate(length(min = 1, max = "FEATURE_TEXT_MAX"))]
    #[schema(min_length = 1, max_length = 200)]
    pub text: String,
}

/// Editable homepage blocks (the "Latest news" list comes from `/news`).
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct HomepageContent {
    #[validate(length(min = 1, max = "HERO_HEADING_MAX"))]
    #[schema(min_length = 1, max_length = 120)]
    pub hero_heading: String,
    #[validate(length(min = 1, max = "HERO_BUTTON_MAX"))]
    #[schema(min_length = 1, max_length = 60)]
    pub hero_button_label: String,
    /// Exactly three feature blocks.
    #[validate(length(min = "FEATURE_COUNT", max = "FEATURE_COUNT"), nested)]
    #[schema(min_items = 3, max_items = 3)]
    pub features: Vec<HomepageFeature>,
}

/// Site-wide settings. Public: the title and slogan are shown to every visitor.
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SiteSettings {
    #[validate(length(min = 1, max = "SITE_TITLE_MAX"))]
    #[schema(min_length = 1, max_length = 60)]
    pub site_title: String,
    /// Short tagline shown on the homepage.
    #[validate(length(min = 1, max = "SLOGAN_MAX"))]
    #[schema(min_length = 1, max_length = 300)]
    pub slogan: String,
    /// Interface language for visitors and new accounts: `en-US` or `de`.
    #[schema(example = "en-US")]
    pub default_language: String,
}

/// Terms-of-service text (plain text / Markdown).
#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TermsContent {
    #[validate(length(min = 1, max = "TERMS_BODY_MAX"))]
    #[schema(min_length = 1, max_length = 20000)]
    pub body: String,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NewsItem {
    pub id: Uuid,
    pub title: String,
    /// Cosmetic URL slug derived from the title; the `id` is authoritative.
    pub slug: String,
    pub body: String,
    pub author: Option<UserSummary>,
    pub published_at: DateTime<Utc>,
}

page_dto!(
    /// A page of news items, newest first.
    NewsItemPage,
    NewsItem
);

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateNewsRequest {
    #[validate(length(min = "NEWS_TITLE_MIN", max = "NEWS_TITLE_MAX"))]
    #[schema(min_length = 1, max_length = 120)]
    pub title: String,
    #[validate(length(max = "NEWS_BODY_MAX"))]
    #[schema(max_length = 5000)]
    pub body: String,
}

/// Partial update; absent fields are unchanged.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateNewsRequest {
    #[validate(length(min = "NEWS_TITLE_MIN", max = "NEWS_TITLE_MAX"))]
    #[schema(min_length = 1, max_length = 120)]
    pub title: Option<String>,
    #[validate(length(max = "NEWS_BODY_MAX"))]
    #[schema(max_length = 5000)]
    pub body: Option<String>,
}

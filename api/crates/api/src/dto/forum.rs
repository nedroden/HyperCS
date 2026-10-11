use chrono::{DateTime, Utc};
use hypercs_domain::{constants::*, pagination::PageParams};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;
use validator::Validate;

use super::common::{UserSummary, page_dto};
use hypercs_domain::Role;

// ---- Responses ----

/// The newest post of a board or the whole forum.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LastPostSummary {
    pub post_id: Uuid,
    pub topic_id: Uuid,
    pub topic_title: String,
    pub author: UserSummary,
    pub created_at: DateTime<Utc>,
}

/// A board as listed on the board index.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardSummary {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub admin_only: bool,
    pub topic_count: i32,
    pub post_count: i32,
    /// Absent for boards without topics.
    pub last_post: Option<LastPostSummary>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryWithBoards {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub position: i32,
    pub admin_only: bool,
    pub boards: Vec<BoardSummary>,
}

/// The board index. Admin-only categories/boards are only included for admins.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardIndexResponse {
    pub categories: Vec<CategoryWithBoards>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardDetail {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub name: String,
    pub description: String,
    /// Effective flag (the board's own OR its category's).
    pub admin_only: bool,
    pub topic_count: i32,
    pub post_count: i32,
}

/// Last activity of a topic.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicLastPost {
    pub post_id: Uuid,
    pub author: UserSummary,
    pub created_at: DateTime<Utc>,
}

/// A topic row in a board's topic list.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicSummary {
    pub id: Uuid,
    pub title: String,
    /// Cosmetic slug for URLs; the `id` is authoritative.
    pub slug: String,
    pub author: UserSummary,
    pub pinned: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
    pub last_post: Option<TopicLastPost>,
}

page_dto!(
    /// A page of topics: pinned first, then by latest activity.
    TopicSummaryPage,
    TopicSummary
);

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardPageResponse {
    pub board: BoardDetail,
    pub topics: TopicSummaryPage,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicDetail {
    pub id: Uuid,
    pub board_id: Uuid,
    pub board_name: String,
    pub category_id: Uuid,
    pub category_name: String,
    pub title: String,
    pub slug: String,
    /// Deleting this post deletes the whole topic.
    pub first_post_id: Uuid,
    pub author: UserSummary,
    pub pinned: bool,
    pub locked: bool,
    pub reply_count: i32,
    pub view_count: i32,
    pub created_at: DateTime<Utc>,
}

/// A post's author with the data shown in the author panel.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostAuthor {
    pub id: Uuid,
    pub username: String,
    pub display_name: String,
    pub role: Role,
    pub post_count: i32,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Post {
    pub id: Uuid,
    pub topic_id: Uuid,
    /// 1-based position within the topic (the post "number" / permalink).
    pub number: i64,
    pub author: PostAuthor,
    /// Raw Markdown as submitted; render it sanitized.
    pub body: String,
    pub created_at: DateTime<Utc>,
    /// Set when the post was edited; the UI shows an "edited" marker.
    pub edited_at: Option<DateTime<Utc>>,
    pub edited_by: Option<UserSummary>,
}

/// Topic, board and category context of a single post.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostTopicContext {
    pub topic_id: Uuid,
    pub topic_title: String,
    pub topic_slug: String,
    pub topic_locked: bool,
    pub board_id: Uuid,
    pub board_name: String,
    pub category_id: Uuid,
    pub category_name: String,
    /// Deleting the first post deletes the whole topic.
    pub is_first_post: bool,
    /// 1-based page of the topic the post is on, at the default page size.
    pub page: u32,
}

/// A single post with the context needed to render its edit/history screens.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostDetail {
    pub post: Post,
    pub context: PostTopicContext,
}

page_dto!(
    /// A page of posts in topic order.
    PostPage,
    Post
);

#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct TopicPageResponse {
    pub topic: TopicDetail,
    pub posts: PostPage,
}

/// The body of a post as it was before an edit.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostRevision {
    pub id: Uuid,
    pub body: String,
    /// Who made the edit that replaced this version.
    pub edited_by: Option<UserSummary>,
    /// When this version was replaced.
    pub created_at: DateTime<Utc>,
}

page_dto!(
    /// A page of a post's edit history, newest first.
    PostRevisionPage,
    PostRevision
);

// ---- Requests ----

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTopicRequest {
    #[validate(length(min = "TOPIC_TITLE_MIN", max = "TOPIC_TITLE_MAX"))]
    #[schema(min_length = 3, max_length = 120)]
    pub title: String,
    /// First post, Markdown.
    #[validate(length(min = "POST_BODY_MIN", max = "POST_BODY_MAX"))]
    #[schema(min_length = 1, max_length = 10000)]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreatePostRequest {
    /// Markdown.
    #[validate(length(min = "POST_BODY_MIN", max = "POST_BODY_MAX"))]
    #[schema(min_length = 1, max_length = 10000)]
    pub body: String,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdatePostRequest {
    /// The new Markdown body. The previous body is kept as a revision.
    #[validate(length(min = "POST_BODY_MIN", max = "POST_BODY_MAX"))]
    #[schema(min_length = 1, max_length = 10000)]
    pub body: String,
}

/// Moderator/admin partial update of a topic. Absent fields are unchanged.
/// Setting `boardId` moves the topic to another board (counters are fixed in
/// the same transaction).
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTopicRequest {
    pub pinned: Option<bool>,
    pub locked: Option<bool>,
    pub board_id: Option<Uuid>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRequest {
    #[validate(length(min = "CATEGORY_BOARD_NAME_MIN", max = "CATEGORY_BOARD_NAME_MAX"))]
    #[schema(min_length = 1, max_length = 60)]
    pub name: String,
    #[serde(default)]
    #[validate(length(max = "DESCRIPTION_MAX"))]
    #[schema(max_length = 200)]
    pub description: String,
    /// Invisible and inaccessible to everyone but admins.
    #[serde(default)]
    pub admin_only: bool,
}

/// Partial update of a category.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCategoryRequest {
    #[validate(length(min = "CATEGORY_BOARD_NAME_MIN", max = "CATEGORY_BOARD_NAME_MAX"))]
    #[schema(min_length = 1, max_length = 60)]
    pub name: Option<String>,
    #[validate(length(max = "DESCRIPTION_MAX"))]
    #[schema(max_length = 200)]
    pub description: Option<String>,
    pub admin_only: Option<bool>,
}

#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BoardRequest {
    pub category_id: Uuid,
    #[validate(length(min = "CATEGORY_BOARD_NAME_MIN", max = "CATEGORY_BOARD_NAME_MAX"))]
    #[schema(min_length = 1, max_length = 60)]
    pub name: String,
    #[serde(default)]
    #[validate(length(max = "DESCRIPTION_MAX"))]
    #[schema(max_length = 200)]
    pub description: String,
    #[serde(default)]
    pub admin_only: bool,
}

/// Partial update of a board.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBoardRequest {
    #[validate(length(min = "CATEGORY_BOARD_NAME_MIN", max = "CATEGORY_BOARD_NAME_MAX"))]
    #[schema(min_length = 1, max_length = 60)]
    pub name: Option<String>,
    #[validate(length(max = "DESCRIPTION_MAX"))]
    #[schema(max_length = 200)]
    pub description: Option<String>,
    pub admin_only: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, ToSchema, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
}

/// Moves a category/board one position up or down among its siblings.
#[derive(Debug, Deserialize, Validate, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReorderRequest {
    pub direction: Direction,
}

// ---- Statistics ----

/// Board-index statistics. `latestPost` only considers boards visible to
/// everyone; `onlineUsers` only lists users who opted in to being shown.
#[derive(Debug, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Stats {
    pub topics: i64,
    pub posts: i64,
    pub members: i64,
    pub boards: i64,
    pub newest_member: Option<UserSummary>,
    pub latest_post: Option<LastPostSummary>,
    /// Users with an authenticated request in the last 5 minutes (those who
    /// opted out of the list are not counted).
    pub online_count: i64,
    /// Up to 50 of the online users, by name.
    pub online_users: Vec<UserSummary>,
}

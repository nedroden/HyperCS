//! Maps database rows to API DTOs.

use hypercs_db::{
    news::NewsItemRow,
    posts::{PostRevisionRow, PostRow},
    structure::{BoardDetailRow, BoardIndexRow, LastPostRow},
    topics::{TopicDetailRow, TopicListRow},
    users::{AdminUserRow, MeRow, PublicProfileRow},
};
use hypercs_domain::Role;

use crate::dto::{
    AdminUser, BoardDetail, BoardSummary, LastPostSummary, MeResponse, NewsItem, Post, PostAuthor,
    PostRevision, PublicProfile, TopicDetail, TopicLastPost, TopicSummary, UserSummary,
};

pub fn me(row: MeRow, csrf_token: String) -> MeResponse {
    MeResponse {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        role: Role::from_db_or_member(&row.role),
        first_name: row.first_name,
        last_name: row.last_name,
        location: row.location,
        website: row.website,
        about_me: row.about_me,
        language: row.language,
        timezone: row.timezone,
        notify_on_reply: row.notify_on_reply,
        notify_on_quote: row.notify_on_quote,
        show_online: row.show_online,
        post_count: row.post_count,
        created_at: row.created_at,
        csrf_token,
    }
}

pub fn public_profile(row: PublicProfileRow) -> PublicProfile {
    PublicProfile {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        role: Role::from_db_or_member(&row.role),
        location: row.location,
        website: row.website,
        about_me: row.about_me,
        post_count: row.post_count,
        created_at: row.created_at,
    }
}

pub fn admin_user(row: AdminUserRow) -> AdminUser {
    AdminUser {
        id: row.id,
        username: row.username,
        display_name: row.display_name,
        email: row.email,
        role: Role::from_db_or_member(&row.role),
        post_count: row.post_count,
        created_at: row.created_at,
    }
}

pub fn last_post(row: LastPostRow) -> LastPostSummary {
    LastPostSummary {
        post_id: row.post_id,
        topic_id: row.topic_id,
        topic_title: row.topic_title,
        author: UserSummary::new(
            row.author_id,
            row.author_username,
            row.author_display_name,
            &row.author_role,
        ),
        created_at: row.created_at,
    }
}

pub fn board_summary(row: BoardIndexRow) -> BoardSummary {
    let last_post = match (
        row.lp_post_id,
        row.lp_topic_id,
        row.lp_topic_title,
        row.lp_created_at,
        UserSummary::from_joined(
            row.lp_author_id,
            row.lp_author_username,
            row.lp_author_display_name,
            row.lp_author_role,
        ),
    ) {
        (Some(post_id), Some(topic_id), Some(topic_title), Some(created_at), Some(author)) => {
            Some(LastPostSummary {
                post_id,
                topic_id,
                topic_title,
                author,
                created_at,
            })
        }
        _ => None,
    };
    BoardSummary {
        id: row.id,
        category_id: row.category_id,
        name: row.name,
        description: row.description,
        position: row.position,
        admin_only: row.admin_only,
        topic_count: row.topic_count,
        post_count: row.post_count,
        last_post,
    }
}

pub fn board_detail(row: BoardDetailRow) -> BoardDetail {
    BoardDetail {
        id: row.id,
        category_id: row.category_id,
        category_name: row.category_name,
        name: row.name,
        description: row.description,
        admin_only: row.admin_only,
        topic_count: row.topic_count,
        post_count: row.post_count,
    }
}

pub fn topic_summary(row: TopicListRow) -> TopicSummary {
    let last_post = match (
        row.lp_post_id,
        row.lp_created_at,
        UserSummary::from_joined(
            row.lp_author_id,
            row.lp_author_username,
            row.lp_author_display_name,
            row.lp_author_role,
        ),
    ) {
        (Some(post_id), Some(created_at), Some(author)) => Some(TopicLastPost {
            post_id,
            author,
            created_at,
        }),
        _ => None,
    };
    TopicSummary {
        id: row.id,
        title: row.title,
        slug: row.slug,
        author: UserSummary::new(
            row.author_id,
            row.author_username,
            row.author_display_name,
            &row.author_role,
        ),
        pinned: row.pinned,
        locked: row.locked,
        reply_count: row.reply_count,
        view_count: row.view_count,
        created_at: row.created_at,
        last_post,
    }
}

pub fn topic_detail(row: TopicDetailRow) -> TopicDetail {
    TopicDetail {
        id: row.id,
        board_id: row.board_id,
        board_name: row.board_name,
        category_id: row.category_id,
        category_name: row.category_name,
        title: row.title,
        slug: row.slug,
        first_post_id: row.first_post_id,
        author: UserSummary::new(
            row.author_id,
            row.author_username,
            row.author_display_name,
            &row.author_role,
        ),
        pinned: row.pinned,
        locked: row.locked,
        reply_count: row.reply_count,
        view_count: row.view_count,
        created_at: row.created_at,
    }
}

pub fn post(row: PostRow, number: i64) -> Post {
    Post {
        id: row.id,
        topic_id: row.topic_id,
        number,
        author: PostAuthor {
            id: row.author_id,
            username: row.author_username,
            display_name: row.author_display_name,
            role: Role::from_db_or_member(&row.author_role),
            post_count: row.author_post_count,
            joined_at: row.author_joined_at,
        },
        body: row.body,
        created_at: row.created_at,
        edited_at: row.edited_at,
        edited_by: UserSummary::from_joined(
            row.editor_id,
            row.editor_username,
            row.editor_display_name,
            row.editor_role,
        ),
    }
}

pub fn revision(row: PostRevisionRow) -> PostRevision {
    PostRevision {
        id: row.id,
        body: row.body,
        edited_by: UserSummary::from_joined(
            row.editor_id,
            row.editor_username,
            row.editor_display_name,
            row.editor_role,
        ),
        created_at: row.created_at,
    }
}

pub fn news_item(row: NewsItemRow) -> NewsItem {
    NewsItem {
        id: row.id,
        title: row.title,
        slug: row.slug,
        body: row.body,
        author: UserSummary::from_joined(
            row.author_id,
            row.author_username,
            row.author_display_name,
            row.author_role,
        ),
        published_at: row.published_at,
    }
}

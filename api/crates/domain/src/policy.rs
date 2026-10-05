//! Central authorization policy. Pure, dependency-free functions so they can
//! be exhaustively unit tested; callers (services) pass in just enough
//! context. Extractors only authenticate; they never decide authorization.

use crate::models::{AuthUser, Role};
use uuid::Uuid;

/// Guests and members may view non-admin-only boards; admin-only boards and
/// categories are invisible and inaccessible to everyone else.
pub fn can_view_board(user: Option<&AuthUser>, board_admin_only: bool) -> bool {
    if !board_admin_only {
        return true;
    }
    matches!(user, Some(u) if u.role == Role::Admin)
}

/// Same rule applies to categories (a category can be admin-only even if a
/// board underneath isn't explicitly, but in practice boards inherit it).
pub fn can_view_category(user: Option<&AuthUser>, category_admin_only: bool) -> bool {
    can_view_board(user, category_admin_only)
}

/// Creating a topic requires a member-or-above account and board visibility.
pub fn can_create_topic(user: Option<&AuthUser>, board_admin_only: bool) -> bool {
    match user {
        Some(u) => can_view_board(Some(u), board_admin_only),
        None => false,
    }
}

/// Replying requires an account; locked topics only accept replies from
/// moderators and admins.
pub fn can_reply_to_topic(
    user: Option<&AuthUser>,
    board_admin_only: bool,
    topic_locked: bool,
) -> bool {
    let Some(u) = user else { return false };
    if !can_view_board(Some(u), board_admin_only) {
        return false;
    }
    if topic_locked {
        return u.role.is_staff();
    }
    true
}

pub fn can_edit_post(user: &AuthUser, post_author_id: Uuid) -> bool {
    user.id == post_author_id || user.role.is_staff()
}

pub fn can_delete_post(user: &AuthUser, post_author_id: Uuid) -> bool {
    user.id == post_author_id || user.role.is_staff()
}

pub fn can_view_post_history(user: &AuthUser, post_author_id: Uuid) -> bool {
    user.id == post_author_id || user.role.is_staff()
}

/// Pin/unpin, lock/unlock, move: moderators and admins only.
pub fn can_moderate_topic(user: &AuthUser) -> bool {
    user.role.is_staff()
}

/// Managing categories/boards, CMS content and roles: admins only.
pub fn can_manage_structure(user: &AuthUser) -> bool {
    user.role == Role::Admin
}

pub fn can_manage_users(user: &AuthUser) -> bool {
    user.role == Role::Admin
}

pub fn can_manage_cms(user: &AuthUser) -> bool {
    user.role == Role::Admin
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(role: Role) -> AuthUser {
        AuthUser {
            id: Uuid::nil(),
            role,
        }
    }

    #[test]
    fn guests_can_view_public_boards_not_admin_only() {
        assert!(can_view_board(None, false));
        assert!(!can_view_board(None, true));
    }

    #[test]
    fn only_admins_view_admin_only_boards() {
        assert!(can_view_board(Some(&user(Role::Admin)), true));
        assert!(!can_view_board(Some(&user(Role::Moderator)), true));
        assert!(!can_view_board(Some(&user(Role::Member)), true));
    }

    #[test]
    fn guests_cannot_create_topics_or_reply() {
        assert!(!can_create_topic(None, false));
        assert!(!can_reply_to_topic(None, false, false));
    }

    #[test]
    fn members_can_create_and_reply_unless_locked() {
        let m = user(Role::Member);
        assert!(can_create_topic(Some(&m), false));
        assert!(can_reply_to_topic(Some(&m), false, false));
        assert!(!can_reply_to_topic(Some(&m), false, true));
    }

    #[test]
    fn staff_can_reply_to_locked_topics() {
        assert!(can_reply_to_topic(
            Some(&user(Role::Moderator)),
            false,
            true
        ));
        assert!(can_reply_to_topic(Some(&user(Role::Admin)), false, true));
    }

    #[test]
    fn edit_delete_post_author_or_staff_only() {
        let author_id = Uuid::new_v4();
        let author = AuthUser {
            id: author_id,
            role: Role::Member,
        };
        let other = AuthUser {
            id: Uuid::new_v4(),
            role: Role::Member,
        };
        let mod_ = user(Role::Moderator);
        let admin = user(Role::Admin);

        assert!(can_edit_post(&author, author_id));
        assert!(!can_edit_post(&other, author_id));
        assert!(can_edit_post(&mod_, author_id));
        assert!(can_edit_post(&admin, author_id));

        assert!(can_delete_post(&author, author_id));
        assert!(!can_delete_post(&other, author_id));
        assert!(can_delete_post(&mod_, author_id));
    }

    #[test]
    fn only_staff_moderate_topics() {
        assert!(!can_moderate_topic(&user(Role::Member)));
        assert!(can_moderate_topic(&user(Role::Moderator)));
        assert!(can_moderate_topic(&user(Role::Admin)));
    }

    #[test]
    fn only_admins_manage_structure_users_cms() {
        assert!(!can_manage_structure(&user(Role::Moderator)));
        assert!(can_manage_structure(&user(Role::Admin)));
        assert!(!can_manage_users(&user(Role::Moderator)));
        assert!(can_manage_users(&user(Role::Admin)));
        assert!(!can_manage_cms(&user(Role::Moderator)));
        assert!(can_manage_cms(&user(Role::Admin)));
    }
}

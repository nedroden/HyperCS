//! Central authorization policy. Pure functions so they can be exhaustively
//! unit tested; services pass in just enough context. Extractors only
//! authenticate; they never decide authorization.
//!
//! `board_admin_only` is always the *effective* flag: the board's own flag OR
//! its category's flag.

use crate::models::{AuthUser, Role};
use uuid::Uuid;

/// Guests, members and moderators may view non-admin-only boards;
/// admin-only boards and categories are invisible and inaccessible to
/// everyone but admins.
pub fn can_view_board(user: Option<&AuthUser>, board_admin_only: bool) -> bool {
    !board_admin_only || matches!(user, Some(u) if u.role == Role::Admin)
}

/// Categories follow the same rule as boards.
pub fn can_view_category(user: Option<&AuthUser>, category_admin_only: bool) -> bool {
    can_view_board(user, category_admin_only)
}

/// Creating a topic requires an account and visibility of the board.
pub fn can_create_topic(user: Option<&AuthUser>, board_admin_only: bool) -> bool {
    user.is_some_and(|u| can_view_board(Some(u), board_admin_only))
}

/// Replying requires an account and board visibility; locked topics only
/// accept replies from moderators and admins.
pub fn can_reply_to_topic(
    user: Option<&AuthUser>,
    board_admin_only: bool,
    topic_locked: bool,
) -> bool {
    let Some(u) = user else { return false };
    can_view_board(Some(u), board_admin_only) && (!topic_locked || u.role.is_staff())
}

fn is_author_or_staff(user: &AuthUser, post_author_id: Uuid) -> bool {
    user.id == post_author_id || user.role.is_staff()
}

pub fn can_edit_post(user: &AuthUser, post_author_id: Uuid, board_admin_only: bool) -> bool {
    can_view_board(Some(user), board_admin_only) && is_author_or_staff(user, post_author_id)
}

pub fn can_delete_post(user: &AuthUser, post_author_id: Uuid, board_admin_only: bool) -> bool {
    can_view_board(Some(user), board_admin_only) && is_author_or_staff(user, post_author_id)
}

pub fn can_view_post_history(
    user: &AuthUser,
    post_author_id: Uuid,
    board_admin_only: bool,
) -> bool {
    can_view_board(Some(user), board_admin_only) && is_author_or_staff(user, post_author_id)
}

/// Pin/unpin, lock/unlock, move and delete a topic: moderators and admins,
/// and only for boards they can see.
pub fn can_moderate_topic(user: &AuthUser, board_admin_only: bool) -> bool {
    user.role.is_staff() && can_view_board(Some(user), board_admin_only)
}

/// Managing categories/boards: admins only.
pub fn can_manage_structure(user: &AuthUser) -> bool {
    user.role == Role::Admin
}

/// Listing users and assigning roles: admins only.
pub fn can_manage_users(user: &AuthUser) -> bool {
    user.role == Role::Admin
}

/// Editing homepage, news and terms: admins only.
pub fn can_manage_cms(user: &AuthUser) -> bool {
    user.role == Role::Admin
}

/// True if changing `target_role` to `new_role` would leave the site without
/// any admin. `admin_count` is the number of admins before the change.
pub fn would_remove_last_admin(target_role: Role, new_role: Role, admin_count: i64) -> bool {
    target_role == Role::Admin && new_role != Role::Admin && admin_count <= 1
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user(role: Role) -> AuthUser {
        AuthUser {
            id: Uuid::from_u128(1),
            role,
        }
    }

    fn other_author() -> Uuid {
        Uuid::from_u128(2)
    }

    /// Every principal: a guest plus one user per role.
    fn principals() -> Vec<(&'static str, Option<AuthUser>)> {
        let mut v = vec![("guest", None)];
        for role in Role::ALL {
            v.push((role.as_db_str(), Some(user(role))));
        }
        v
    }

    #[test]
    fn view_board_matrix() {
        for (name, p) in principals() {
            // public boards: everyone
            assert!(can_view_board(p.as_ref(), false), "{name} public");
            // admin-only boards: admins only
            assert_eq!(
                can_view_board(p.as_ref(), true),
                name == "admin",
                "{name} admin-only"
            );
            assert_eq!(
                can_view_category(p.as_ref(), true),
                can_view_board(p.as_ref(), true)
            );
        }
    }

    #[test]
    fn create_topic_matrix() {
        for (name, p) in principals() {
            let logged_in = p.is_some();
            assert_eq!(can_create_topic(p.as_ref(), false), logged_in, "{name}");
            assert_eq!(
                can_create_topic(p.as_ref(), true),
                name == "admin",
                "{name}"
            );
        }
    }

    #[test]
    fn reply_matrix() {
        for (name, p) in principals() {
            let logged_in = p.is_some();
            let staff = matches!(name, "admin" | "moderator");
            // open topic in public board
            assert_eq!(
                can_reply_to_topic(p.as_ref(), false, false),
                logged_in,
                "{name}"
            );
            // locked topic in public board: staff only
            assert_eq!(can_reply_to_topic(p.as_ref(), false, true), staff, "{name}");
            // admin-only board: admins only, locked or not
            assert_eq!(
                can_reply_to_topic(p.as_ref(), true, false),
                name == "admin",
                "{name}"
            );
            assert_eq!(
                can_reply_to_topic(p.as_ref(), true, true),
                name == "admin",
                "{name}"
            );
        }
    }

    #[test]
    fn edit_delete_history_matrix() {
        for role in Role::ALL {
            let actor = user(role);
            let staff = role.is_staff();
            // own post
            for f in [can_edit_post, can_delete_post, can_view_post_history] {
                assert!(f(&actor, actor.id, false), "{role:?} own post");
                // someone else's post: staff only
                assert_eq!(
                    f(&actor, other_author(), false),
                    staff,
                    "{role:?} foreign post"
                );
                // admin-only board: admins only, even for the author
                assert_eq!(
                    f(&actor, actor.id, true),
                    role == Role::Admin,
                    "{role:?} own, admin-only"
                );
                assert_eq!(
                    f(&actor, other_author(), true),
                    role == Role::Admin,
                    "{role:?} foreign, admin-only"
                );
            }
        }
    }

    #[test]
    fn moderate_topic_matrix() {
        for role in Role::ALL {
            let actor = user(role);
            assert_eq!(can_moderate_topic(&actor, false), role.is_staff());
            assert_eq!(can_moderate_topic(&actor, true), role == Role::Admin);
        }
    }

    #[test]
    fn admin_only_capabilities_matrix() {
        for role in Role::ALL {
            let actor = user(role);
            let admin = role == Role::Admin;
            assert_eq!(can_manage_structure(&actor), admin, "{role:?}");
            assert_eq!(can_manage_users(&actor), admin, "{role:?}");
            assert_eq!(can_manage_cms(&actor), admin, "{role:?}");
        }
    }

    #[test]
    fn last_admin_cannot_be_demoted() {
        for new_role in Role::ALL {
            let demotes = new_role != Role::Admin;
            assert_eq!(would_remove_last_admin(Role::Admin, new_role, 1), demotes);
            assert!(!would_remove_last_admin(Role::Admin, new_role, 2));
            assert!(!would_remove_last_admin(Role::Member, new_role, 1));
            assert!(!would_remove_last_admin(Role::Moderator, new_role, 1));
        }
    }

    #[test]
    fn role_helpers() {
        assert!(Role::Admin.is_staff() && Role::Moderator.is_staff() && !Role::Member.is_staff());
        for role in Role::ALL {
            assert_eq!(Role::from_db_str(role.as_db_str()), Some(role));
        }
        assert_eq!(Role::from_db_str("root"), None);
        assert_eq!(Role::from_db_or_member("root"), Role::Member);
    }
}

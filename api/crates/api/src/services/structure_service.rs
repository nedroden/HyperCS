use std::collections::HashMap;

use hypercs_db::PgPool;
use hypercs_domain::{
    AuthUser as Principal, Role,
    constants::{
        CATEGORY_BOARD_NAME_MAX, CATEGORY_BOARD_NAME_MIN, DESCRIPTION_MAX, TOPICS_PER_PAGE,
    },
    pagination::PageParams,
    policy,
};
use uuid::Uuid;

use crate::{
    dto::{
        BoardIndexResponse, BoardPageResponse, BoardRequest, CategoryRequest, CategoryWithBoards,
        Direction, TopicSummaryPage, UpdateBoardRequest, UpdateCategoryRequest,
    },
    error::{ApiError, ApiResult},
    services::{text, view},
};

fn require_admin(user: &Principal) -> ApiResult<()> {
    if policy::can_manage_structure(user) {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}

fn name(input: &str) -> ApiResult<String> {
    text::line(
        "name",
        input,
        CATEGORY_BOARD_NAME_MIN,
        CATEGORY_BOARD_NAME_MAX,
    )
}

fn description(input: &str) -> ApiResult<String> {
    text::text_trimmed("description", input, DESCRIPTION_MAX)
}

/// The board index. Admin-only categories and boards are omitted for
/// everyone but admins.
pub async fn board_index(db: &PgPool, viewer: Option<&Principal>) -> ApiResult<BoardIndexResponse> {
    let include_admin_only = viewer.is_some_and(|u| u.role == Role::Admin);
    let categories = hypercs_db::structure::list_categories(db, include_admin_only).await?;
    let board_rows = hypercs_db::structure::list_board_index_rows(db, include_admin_only).await?;

    let mut by_category: HashMap<Uuid, Vec<_>> = HashMap::new();
    for row in board_rows {
        by_category
            .entry(row.category_id)
            .or_default()
            .push(view::board_summary(row));
    }
    Ok(BoardIndexResponse {
        categories: categories
            .into_iter()
            .map(|c| CategoryWithBoards {
                boards: by_category.remove(&c.id).unwrap_or_default(),
                id: c.id,
                name: c.name,
                description: c.description,
                position: c.position,
                admin_only: c.admin_only,
            })
            .collect(),
    })
}

pub async fn board_page(
    db: &PgPool,
    board_id: Uuid,
    viewer: Option<&Principal>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> ApiResult<BoardPageResponse> {
    let board = hypercs_db::structure::get_board_detail(db, board_id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    if !policy::can_view_board(viewer, board.admin_only) {
        return Err(ApiError::not_found());
    }
    let params = PageParams::new(page, per_page, TOPICS_PER_PAGE);
    let rows =
        hypercs_db::topics::list_for_board(db, board_id, params.limit(), params.offset()).await?;
    let total = hypercs_db::topics::count_for_board(db, board_id).await?;
    Ok(BoardPageResponse {
        board: view::board_detail(board),
        topics: TopicSummaryPage::new(
            rows.into_iter().map(view::topic_summary).collect(),
            params,
            total,
        ),
    })
}

pub async fn create_category(
    db: &PgPool,
    user: &Principal,
    req: &CategoryRequest,
) -> ApiResult<Uuid> {
    require_admin(user)?;
    let name = name(&req.name)?;
    let description = description(&req.description)?;
    let position = hypercs_db::structure::next_category_position(db).await?;
    let id = Uuid::now_v7();
    hypercs_db::structure::create_category(db, id, &name, &description, req.admin_only, position)
        .await?;
    Ok(id)
}

pub async fn update_category(
    db: &PgPool,
    user: &Principal,
    id: Uuid,
    req: &UpdateCategoryRequest,
) -> ApiResult<()> {
    require_admin(user)?;
    let mut tx = db.begin().await?;
    let current = hypercs_db::structure::get_category_for_update(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let new_name = match &req.name {
        Some(n) => name(n)?,
        None => current.name,
    };
    let new_description = match &req.description {
        Some(d) => description(d)?,
        None => current.description,
    };
    hypercs_db::structure::update_category(
        &mut *tx,
        id,
        &new_name,
        &new_description,
        req.admin_only.unwrap_or(current.admin_only),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Hard-deletes the category with all its boards, topics and posts; authors'
/// post counts are corrected in the same transaction.
pub async fn delete_category(db: &PgPool, user: &Principal, id: Uuid) -> ApiResult<()> {
    require_admin(user)?;
    let mut tx = db.begin().await?;
    hypercs_db::structure::get_category_for_update(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    hypercs_db::structure::decrement_author_counts_for_category(&mut *tx, id).await?;
    hypercs_db::structure::delete_category(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn reorder_category(
    db: &PgPool,
    user: &Principal,
    id: Uuid,
    direction: Direction,
) -> ApiResult<()> {
    require_admin(user)?;
    let mut tx = db.begin().await?;
    let mut categories = hypercs_db::structure::list_categories_ordered_for_update(&mut tx).await?;
    let index = categories
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(ApiError::not_found)?;
    if let Some(neighbor) = neighbor_index(index, categories.len(), direction) {
        categories.swap(index, neighbor);
        for (i, category) in categories.iter().enumerate() {
            hypercs_db::structure::set_category_position(&mut *tx, category.id, i as i32).await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

fn neighbor_index(index: usize, len: usize, direction: Direction) -> Option<usize> {
    match direction {
        Direction::Up => index.checked_sub(1),
        Direction::Down => (index + 1 < len).then_some(index + 1),
    }
}

pub async fn create_board(db: &PgPool, user: &Principal, req: &BoardRequest) -> ApiResult<Uuid> {
    require_admin(user)?;
    let name = name(&req.name)?;
    let description = description(&req.description)?;
    let mut tx = db.begin().await?;
    if hypercs_db::structure::get_category_for_update(&mut tx, req.category_id)
        .await?
        .is_none()
    {
        return Err(ApiError::validation(
            "categoryId",
            "category_not_found",
            "The category does not exist.",
        ));
    }
    let position = hypercs_db::structure::next_board_position(&mut *tx, req.category_id).await?;
    let id = Uuid::now_v7();
    hypercs_db::structure::create_board(
        &mut *tx,
        id,
        req.category_id,
        &name,
        &description,
        req.admin_only,
        position,
    )
    .await?;
    tx.commit().await?;
    Ok(id)
}

pub async fn update_board(
    db: &PgPool,
    user: &Principal,
    id: Uuid,
    req: &UpdateBoardRequest,
) -> ApiResult<()> {
    require_admin(user)?;
    let mut tx = db.begin().await?;
    let current = hypercs_db::structure::get_board_for_update(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let new_name = match &req.name {
        Some(n) => name(n)?,
        None => current.name,
    };
    let new_description = match &req.description {
        Some(d) => description(d)?,
        None => current.description,
    };
    hypercs_db::structure::update_board(
        &mut *tx,
        id,
        &new_name,
        &new_description,
        req.admin_only.unwrap_or(current.admin_only),
    )
    .await?;
    tx.commit().await?;
    Ok(())
}

/// Hard-deletes the board with its topics and posts; authors' post counts
/// are corrected in the same transaction.
pub async fn delete_board(db: &PgPool, user: &Principal, id: Uuid) -> ApiResult<()> {
    require_admin(user)?;
    let mut tx = db.begin().await?;
    hypercs_db::structure::get_board_for_update(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    hypercs_db::structure::decrement_author_counts_for_board(&mut *tx, id).await?;
    hypercs_db::structure::delete_board(&mut *tx, id).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn reorder_board(
    db: &PgPool,
    user: &Principal,
    id: Uuid,
    direction: Direction,
) -> ApiResult<()> {
    require_admin(user)?;
    let mut tx = db.begin().await?;
    let board = hypercs_db::structure::get_board_for_update(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let mut boards =
        hypercs_db::structure::list_boards_ordered_for_update(&mut tx, board.category_id).await?;
    let index = boards
        .iter()
        .position(|b| b.id == id)
        .ok_or_else(ApiError::not_found)?;
    if let Some(neighbor) = neighbor_index(index, boards.len(), direction) {
        boards.swap(index, neighbor);
        for (i, b) in boards.iter().enumerate() {
            hypercs_db::structure::set_board_position(&mut *tx, b.id, i as i32).await?;
        }
    }
    tx.commit().await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbors() {
        assert_eq!(neighbor_index(0, 3, Direction::Up), None);
        assert_eq!(neighbor_index(1, 3, Direction::Up), Some(0));
        assert_eq!(neighbor_index(1, 3, Direction::Down), Some(2));
        assert_eq!(neighbor_index(2, 3, Direction::Down), None);
        assert_eq!(neighbor_index(0, 1, Direction::Down), None);
    }
}

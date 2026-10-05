use hypercs_db::PgPool;
use hypercs_db::pagination::{Page, PageParams};
use hypercs_domain::constants::TOPICS_PER_PAGE;
use hypercs_domain::validation::normalize_text;
use hypercs_domain::{AuthUser, DomainError, DomainResult};
use uuid::Uuid;

use crate::dto::{BoardRequest, CategoryRequest, Direction};

pub async fn board_index(
    db: &PgPool,
    viewer: Option<&AuthUser>,
) -> DomainResult<Vec<hypercs_db::structure::CategoryIndexItem>> {
    let include_admin_only = matches!(viewer, Some(u) if u.role == hypercs_domain::Role::Admin);
    hypercs_db::structure::list_board_index(db, include_admin_only)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn board_detail(
    db: &PgPool,
    board_id: Uuid,
    viewer: Option<&AuthUser>,
    page: Option<u32>,
    per_page: Option<u32>,
) -> DomainResult<(
    hypercs_db::structure::BoardDetailRow,
    Page<hypercs_db::topics::TopicListItemRow>,
)> {
    let board = hypercs_db::structure::get_board_detail(db, board_id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    if !hypercs_domain::policy::can_view_board(viewer, board.admin_only) {
        return Err(DomainError::NotFound);
    }
    let params = PageParams::new(page, per_page, TOPICS_PER_PAGE);
    let items = hypercs_db::topics::list_for_board(db, board_id, params.limit(), params.offset())
        .await
        .map_err(|_| DomainError::Internal)?;
    let total = hypercs_db::topics::count_for_board(db, board_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    Ok((board, Page::new(items, params, total)))
}

fn require_admin(user: &AuthUser) -> DomainResult<()> {
    if hypercs_domain::policy::can_manage_structure(user) {
        Ok(())
    } else {
        Err(DomainError::Forbidden)
    }
}

pub async fn create_category(
    db: &PgPool,
    user: &AuthUser,
    req: &CategoryRequest,
) -> DomainResult<Uuid> {
    require_admin(user)?;
    let name = normalize_text(&req.name, false);
    let description = normalize_text(&req.description, false);
    let position = hypercs_db::structure::next_category_position(db)
        .await
        .map_err(|_| DomainError::Internal)?;
    let id = Uuid::now_v7();
    hypercs_db::structure::create_category(db, id, &name, &description, req.admin_only, position)
        .await
        .map_err(|_| DomainError::Internal)?;
    Ok(id)
}

pub async fn update_category(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    req: &CategoryRequest,
) -> DomainResult<()> {
    require_admin(user)?;
    let name = normalize_text(&req.name, false);
    let description = normalize_text(&req.description, false);
    hypercs_db::structure::update_category(db, id, &name, &description, req.admin_only)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn delete_category(db: &PgPool, user: &AuthUser, id: Uuid) -> DomainResult<()> {
    require_admin(user)?;
    let affected = hypercs_db::structure::delete_category(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;
    if affected == 0 {
        return Err(DomainError::NotFound);
    }
    Ok(())
}

pub async fn reorder_category(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    direction: Direction,
) -> DomainResult<()> {
    require_admin(user)?;
    let mut categories = hypercs_db::structure::list_categories_ordered(db)
        .await
        .map_err(|_| DomainError::Internal)?;
    let index = categories
        .iter()
        .position(|c| c.id == id)
        .ok_or(DomainError::NotFound)?;
    let neighbor_index = match direction {
        Direction::Up => index.checked_sub(1),
        Direction::Down => (index + 1 < categories.len()).then_some(index + 1),
    };
    let Some(neighbor_index) = neighbor_index else {
        return Ok(()); // already at the edge; no-op
    };
    categories.swap(index, neighbor_index);
    for (i, category) in categories.iter().enumerate() {
        hypercs_db::structure::set_category_position(db, category.id, i as i32)
            .await
            .map_err(|_| DomainError::Internal)?;
    }
    Ok(())
}

pub async fn create_board(db: &PgPool, user: &AuthUser, req: &BoardRequest) -> DomainResult<Uuid> {
    require_admin(user)?;
    let name = normalize_text(&req.name, false);
    let description = normalize_text(&req.description, false);
    let position = hypercs_db::structure::next_board_position(db, req.category_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    let id = Uuid::now_v7();
    hypercs_db::structure::create_board(
        db,
        id,
        req.category_id,
        &name,
        &description,
        req.admin_only,
        position,
    )
    .await
    .map_err(|_| DomainError::Internal)?;
    Ok(id)
}

pub async fn update_board(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    req: &BoardRequest,
) -> DomainResult<()> {
    require_admin(user)?;
    let name = normalize_text(&req.name, false);
    let description = normalize_text(&req.description, false);
    hypercs_db::structure::update_board(db, id, &name, &description, req.admin_only)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn delete_board(db: &PgPool, user: &AuthUser, id: Uuid) -> DomainResult<()> {
    require_admin(user)?;
    let affected = hypercs_db::structure::delete_board(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;
    if affected == 0 {
        return Err(DomainError::NotFound);
    }
    Ok(())
}

pub async fn reorder_board(
    db: &PgPool,
    user: &AuthUser,
    id: Uuid,
    direction: Direction,
) -> DomainResult<()> {
    require_admin(user)?;
    let board = hypercs_db::structure::get_board_detail(db, id)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)?;
    let mut boards = hypercs_db::structure::list_boards_ordered(db, board.category_id)
        .await
        .map_err(|_| DomainError::Internal)?;
    let index = boards
        .iter()
        .position(|b| b.id == id)
        .ok_or(DomainError::NotFound)?;
    let neighbor_index = match direction {
        Direction::Up => index.checked_sub(1),
        Direction::Down => (index + 1 < boards.len()).then_some(index + 1),
    };
    let Some(neighbor_index) = neighbor_index else {
        return Ok(());
    };
    boards.swap(index, neighbor_index);
    for (i, b) in boards.iter().enumerate() {
        hypercs_db::structure::set_board_position(db, b.id, i as i32)
            .await
            .map_err(|_| DomainError::Internal)?;
    }
    Ok(())
}

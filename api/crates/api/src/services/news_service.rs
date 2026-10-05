use hypercs_db::PgPool;
use hypercs_db::pagination::{Page, PageParams};
use hypercs_domain::validation::{normalize_text, slugify};
use hypercs_domain::{AuthUser, DomainError, DomainResult};
use uuid::Uuid;

use crate::dto::NewsRequest;

const NEWS_PER_PAGE: u32 = 10;

pub async fn list(
    db: &PgPool,
    page: Option<u32>,
    per_page: Option<u32>,
) -> DomainResult<Page<hypercs_db::news::NewsItemRow>> {
    let params = PageParams::new(page, per_page, NEWS_PER_PAGE);
    let items = hypercs_db::news::list(db, params.limit(), params.offset())
        .await
        .map_err(|_| DomainError::Internal)?;
    let total = hypercs_db::news::count(db)
        .await
        .map_err(|_| DomainError::Internal)?;
    Ok(Page::new(items, params, total))
}

pub async fn get_by_slug(db: &PgPool, slug: &str) -> DomainResult<hypercs_db::news::NewsItemRow> {
    hypercs_db::news::get_by_slug(db, slug)
        .await
        .map_err(|_| DomainError::Internal)?
        .ok_or(DomainError::NotFound)
}

fn require_admin(user: &AuthUser) -> DomainResult<()> {
    if hypercs_domain::policy::can_manage_cms(user) {
        Ok(())
    } else {
        Err(DomainError::Forbidden)
    }
}

pub async fn create(db: &PgPool, user: &AuthUser, req: &NewsRequest) -> DomainResult<Uuid> {
    require_admin(user)?;
    let title = normalize_text(&req.title, false);
    let body = normalize_text(&req.body, true);
    let slug = slugify(&title, 100);
    let id = Uuid::now_v7();
    hypercs_db::news::create(db, id, &title, &slug, &body, user.id)
        .await
        .map_err(|_| DomainError::Internal)?;
    Ok(id)
}

pub async fn update(db: &PgPool, user: &AuthUser, id: Uuid, req: &NewsRequest) -> DomainResult<()> {
    require_admin(user)?;
    let title = normalize_text(&req.title, false);
    let body = normalize_text(&req.body, true);
    let slug = slugify(&title, 100);
    hypercs_db::news::update(db, id, &title, &slug, &body)
        .await
        .map_err(|_| DomainError::Internal)
}

pub async fn delete(db: &PgPool, user: &AuthUser, id: Uuid) -> DomainResult<()> {
    require_admin(user)?;
    let affected = hypercs_db::news::delete(db, id)
        .await
        .map_err(|_| DomainError::Internal)?;
    if affected == 0 {
        return Err(DomainError::NotFound);
    }
    Ok(())
}

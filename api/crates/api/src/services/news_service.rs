use hypercs_db::PgPool;
use hypercs_domain::{
    AuthUser as Principal,
    constants::{NEWS_BODY_MAX, NEWS_PER_PAGE, NEWS_TITLE_MAX, NEWS_TITLE_MIN},
    pagination::PageParams,
    policy,
    validation::slugify,
};
use uuid::Uuid;

use crate::{
    dto::{CreateNewsRequest, NewsItem, NewsItemPage, UpdateNewsRequest},
    error::{ApiError, ApiResult},
    services::{text, view},
};

const SLUG_MAX: usize = 100;

pub async fn list(
    db: &PgPool,
    page: Option<u32>,
    per_page: Option<u32>,
) -> ApiResult<NewsItemPage> {
    let params = PageParams::new(page, per_page, NEWS_PER_PAGE);
    let rows = hypercs_db::news::list(db, params.limit(), params.offset()).await?;
    let total = hypercs_db::news::count(db).await?;
    Ok(NewsItemPage::new(
        rows.into_iter().map(view::news_item).collect(),
        params,
        total,
    ))
}

pub async fn get(db: &PgPool, id: Uuid) -> ApiResult<NewsItem> {
    hypercs_db::news::get(db, id)
        .await?
        .map(view::news_item)
        .ok_or_else(ApiError::not_found)
}

fn require_admin(user: &Principal) -> ApiResult<()> {
    if policy::can_manage_cms(user) {
        Ok(())
    } else {
        Err(ApiError::forbidden())
    }
}

fn title(input: &str) -> ApiResult<String> {
    text::line("title", input, NEWS_TITLE_MIN, NEWS_TITLE_MAX)
}

fn body(input: &str) -> ApiResult<String> {
    text::body("body", input, 0, NEWS_BODY_MAX)
}

pub async fn create(db: &PgPool, user: &Principal, req: &CreateNewsRequest) -> ApiResult<Uuid> {
    require_admin(user)?;
    let title = title(&req.title)?;
    let body = body(&req.body)?;
    let slug = slugify(&title, SLUG_MAX, "news");
    let id = Uuid::now_v7();
    hypercs_db::news::create(db, id, &title, &slug, &body, user.id).await?;
    Ok(id)
}

pub async fn update(
    db: &PgPool,
    user: &Principal,
    id: Uuid,
    req: &UpdateNewsRequest,
) -> ApiResult<()> {
    require_admin(user)?;
    let mut tx = db.begin().await?;
    let (current_title, current_body) = hypercs_db::news::get_for_update(&mut tx, id)
        .await?
        .ok_or_else(ApiError::not_found)?;
    let new_title = match &req.title {
        Some(t) => title(t)?,
        None => current_title,
    };
    let new_body = match &req.body {
        Some(b) => body(b)?,
        None => current_body,
    };
    let slug = slugify(&new_title, SLUG_MAX, "news");
    hypercs_db::news::update(&mut *tx, id, &new_title, &slug, &new_body).await?;
    tx.commit().await?;
    Ok(())
}

pub async fn delete(db: &PgPool, user: &Principal, id: Uuid) -> ApiResult<()> {
    require_admin(user)?;
    if hypercs_db::news::delete(db, id).await? == 0 {
        return Err(ApiError::not_found());
    }
    Ok(())
}

use hypercs_domain::constants::MAX_PAGE_SIZE;
use serde::Serialize;
use utoipa::ToSchema;

/// Page-number pagination parameters as received from the client, already
/// clamped to sane bounds.
#[derive(Debug, Clone, Copy)]
pub struct PageParams {
    pub page: u32,
    pub per_page: u32,
}

impl PageParams {
    /// `page` is 1-based; `requested_per_page` is clamped to
    /// `[1, MAX_PAGE_SIZE]`, falling back to `default_per_page` when absent.
    pub fn new(page: Option<u32>, per_page: Option<u32>, default_per_page: u32) -> Self {
        let page = page.unwrap_or(1).max(1);
        let per_page = per_page.unwrap_or(default_per_page).clamp(1, MAX_PAGE_SIZE);
        Self { page, per_page }
    }

    pub fn offset(&self) -> i64 {
        ((self.page - 1) as i64) * (self.per_page as i64)
    }

    pub fn limit(&self) -> i64 {
        self.per_page as i64
    }
}

#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub per_page: u32,
    pub total: i64,
    pub total_pages: u32,
}

impl<T> Page<T> {
    pub fn new(items: Vec<T>, params: PageParams, total: i64) -> Self {
        let total_pages = if total <= 0 {
            0
        } else {
            ((total as u64).div_ceil(params.per_page as u64)) as u32
        };
        Self {
            items,
            page: params.page,
            per_page: params.per_page,
            total,
            total_pages,
        }
    }
}

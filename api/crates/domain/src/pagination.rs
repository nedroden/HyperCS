//! Page-number pagination parameters shared by every list endpoint.

use crate::constants::MAX_PAGE_SIZE;

/// Pagination as requested by the client, already clamped to sane bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageParams {
    pub page: u32,
    pub per_page: u32,
}

impl PageParams {
    /// `page` is 1-based. `per_page` is clamped to `[1, MAX_PAGE_SIZE]` and
    /// falls back to `default_per_page` when absent.
    pub fn new(page: Option<u32>, per_page: Option<u32>, default_per_page: u32) -> Self {
        let page = page.unwrap_or(1).clamp(1, 1_000_000);
        let per_page = per_page.unwrap_or(default_per_page).clamp(1, MAX_PAGE_SIZE);
        Self { page, per_page }
    }

    pub fn offset(&self) -> i64 {
        i64::from(self.page - 1) * i64::from(self.per_page)
    }

    pub fn limit(&self) -> i64 {
        i64::from(self.per_page)
    }

    pub fn total_pages(&self, total: i64) -> u32 {
        if total <= 0 {
            0
        } else {
            (total as u64).div_ceil(u64::from(self.per_page)) as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_clamping() {
        let p = PageParams::new(None, None, 20);
        assert_eq!((p.page, p.per_page, p.offset()), (1, 20, 0));
        let p = PageParams::new(Some(0), Some(0), 20);
        assert_eq!((p.page, p.per_page), (1, 1));
        let p = PageParams::new(Some(3), Some(10_000), 20);
        assert_eq!((p.page, p.per_page, p.offset()), (3, MAX_PAGE_SIZE, 200));
    }

    #[test]
    fn total_pages_rounds_up() {
        let p = PageParams::new(None, Some(20), 20);
        assert_eq!(p.total_pages(0), 0);
        assert_eq!(p.total_pages(1), 1);
        assert_eq!(p.total_pages(20), 1);
        assert_eq!(p.total_pages(21), 2);
    }
}

//! Business rules and transactions. Authorization decisions come from
//! `hypercs_domain::policy`; SQL lives in `hypercs-db`.
//!
//! Lock order inside transactions (to avoid deadlocks): per-user posting
//! advisory lock, then the topic row, then board rows (ascending id when
//! several), then user counters.

pub mod auth_service;
pub mod cms_service;
pub mod flood;
pub mod news_service;
pub mod post_service;
pub mod stats_service;
pub mod structure_service;
pub mod text;
pub mod topic_service;
pub mod user_service;
pub mod view;

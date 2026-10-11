//! Request/response DTOs. All JSON is camelCase. Validation limits come from
//! `hypercs_domain::constants` (the `validator` attributes) and are repeated
//! as `#[schema(...)]` attributes so the OpenAPI document carries them; a
//! test in `openapi.rs` asserts the two agree.

mod auth;
mod cms;
mod common;
mod forum;
mod users;

pub use auth::*;
pub use cms::*;
pub use common::*;
pub use forum::*;
pub use users::*;

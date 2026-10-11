//! sqlx repositories. Plain `FromRow` structs only: shaping rows into API
//! responses (and any JSON/OpenAPI concern) belongs to the `hypercs-api`
//! crate. Queries are runtime-checked and always parameterized.

pub mod cms;
pub mod maintenance;
pub mod news;
pub mod posts;
pub mod sessions;
pub mod stats;
pub mod structure;
pub mod topics;
pub mod users;

pub use sqlx::PgPool;

/// The embedded migrations, shared by the server and by tests.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

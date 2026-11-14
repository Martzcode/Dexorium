//! Infrastructure adapters for Dexorium.
//!
//! SQLite repositories, the FTS5 search engine, caching and file-system access
//! live here. The domain and application layers only see ports (traits).

pub mod reference;
pub mod storage;
pub mod user;

pub use reference::{REFERENCE_SCHEMA, SqliteSpeciesRepository};
pub use storage::{REFERENCE_DB_FILE, USER_DB_FILE, reference_db_path, user_db_path};
pub use user::{SqliteUserRepository, USER_SCHEMA};

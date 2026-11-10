//! Infrastructure adapters for Dexorium.
//!
//! SQLite repositories, the FTS5 search engine, caching and file-system access
//! live here. The domain and application layers only see ports (traits).

pub mod storage;

pub use storage::{REFERENCE_DB_FILE, USER_DB_FILE, reference_db_path, user_db_path};

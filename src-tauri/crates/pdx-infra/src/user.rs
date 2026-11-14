//! User database: favourites, and later teams, notes and settings.
//!
//! The user database is deliberately separate from the read-only
//! `reference.db`: user data survives dataset upgrades and can be backed up or
//! reset without touching the reference data.

use std::path::{Path, PathBuf};

use pdx_app::FavoriteRepository;
use pdx_domain::{FavoriteEntry, FavoriteKind};
use rusqlite::{Connection, params};

/// Schema of the user database.
///
/// `entity_type` keeps the table forward compatible with teams, notes and
/// other collections described in the specifications.
pub const USER_SCHEMA: &str = r"
CREATE TABLE IF NOT EXISTS favorite (
    entity_type TEXT NOT NULL,
    entity_id   INTEGER NOT NULL,
    created_at  TEXT NOT NULL,
    PRIMARY KEY (entity_type, entity_id)
) WITHOUT ROWID;

CREATE INDEX IF NOT EXISTS idx_favorite_created
    ON favorite(entity_type, created_at DESC);

PRAGMA user_version = 1;
";

/// SQLite implementation of the user collection port.
pub struct SqliteUserRepository {
    connection: Connection,
}

impl SqliteUserRepository {
    /// Opens (or creates) the user database at `path`.
    ///
    /// # Errors
    ///
    /// Returns a message when the parent directory cannot be created or the
    /// database cannot be opened or migrated.
    pub fn open(path: &Path) -> Result<Self, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "cannot create the user data directory {}: {error}",
                    parent.display()
                )
            })?;
        }

        let connection = Connection::open(path).map_err(user_error)?;
        Self::prepare(connection)
    }

    /// Builds an in-memory user database (tests and browser fallback).
    ///
    /// # Errors
    ///
    /// Returns a message when SQLite rejects the schema.
    pub fn in_memory() -> Result<Self, String> {
        let connection = Connection::open_in_memory().map_err(user_error)?;
        Self::prepare(connection)
    }

    fn prepare(connection: Connection) -> Result<Self, String> {
        connection
            .execute_batch(
                "PRAGMA journal_mode = WAL;\n\
                 PRAGMA synchronous = NORMAL;\n\
                 PRAGMA foreign_keys = ON;",
            )
            .map_err(user_error)?;
        connection
            .execute_batch(USER_SCHEMA)
            .map_err(user_error)?;

        Ok(Self { connection })
    }

    /// Returns the database file path, mostly for diagnostics and tests.
    #[must_use]
    pub fn path(&self) -> Option<PathBuf> {
        self.connection
            .path()
            .map(|path| PathBuf::from(path.to_owned()))
    }

    /// Number of starred entities of one kind.
    ///
    /// # Errors
    ///
    /// Returns a message when the query fails.
    pub fn count(&self, kind: FavoriteKind) -> Result<usize, String> {
        let total: i64 = self
            .connection
            .query_row(
                "SELECT COUNT(*) FROM favorite WHERE entity_type = ?1",
                [kind.as_str()],
                |row| row.get(0),
            )
            .map_err(user_error)?;

        Ok(usize::try_from(total).unwrap_or(usize::MAX))
    }
}

impl FavoriteRepository for SqliteUserRepository {
    fn favorite_entries(&self) -> Result<Vec<FavoriteEntry>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT entity_id, created_at
                   FROM favorite
                  WHERE entity_type = ?1
                  ORDER BY created_at DESC, entity_id ASC",
            )
            .map_err(user_error)?;

        let rows = statement
            .query_map([FavoriteKind::Species.as_str()], |row| {
                Ok(FavoriteEntry {
                    kind: FavoriteKind::Species,
                    entity_id: row.get(0)?,
                    created_at: row.get(1)?,
                })
            })
            .map_err(user_error)?;

        rows.collect::<rusqlite::Result<Vec<_>>>()
            .map_err(user_error)
    }

    fn set_favorite(
        &self,
        kind: FavoriteKind,
        entity_id: i64,
        favorite: bool,
    ) -> Result<(), String> {
        if favorite {
            self.connection
                .execute(
                    "INSERT INTO favorite (entity_type, entity_id, created_at)
                     VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
                     ON CONFLICT (entity_type, entity_id) DO NOTHING",
                    params![kind.as_str(), entity_id],
                )
                .map_err(user_error)?;
        } else {
            self.connection
                .execute(
                    "DELETE FROM favorite WHERE entity_type = ?1 AND entity_id = ?2",
                    params![kind.as_str(), entity_id],
                )
                .map_err(user_error)?;
        }

        Ok(())
    }
}

fn user_error(error: rusqlite::Error) -> String {
    format!("user database error: {error}")
}

#[cfg(test)]
mod tests {
    use pdx_app::FavoriteRepository;
    use pdx_domain::FavoriteKind;

    use super::SqliteUserRepository;

    #[test]
    fn stars_are_idempotent_and_ordered_newest_first() {
        let repository = SqliteUserRepository::in_memory().expect("user database should open");

        repository
            .set_favorite(FavoriteKind::Species, 25, true)
            .expect("starring should succeed");
        repository
            .set_favorite(FavoriteKind::Species, 25, true)
            .expect("starring again should be a no-op");
        repository
            .set_favorite(FavoriteKind::Species, 6, true)
            .expect("starring should succeed");

        let entries = repository.favorite_entries().expect("entries should load");
        assert_eq!(entries.len(), 2);
        assert!(
            entries
                .iter()
                .all(|entry| entry.kind == FavoriteKind::Species)
        );
        assert!(
            entries
                .iter()
                .all(|entry| entry.created_at.ends_with('Z') && entry.created_at.contains('T'))
        );
        assert_eq!(repository.count(FavoriteKind::Species).unwrap(), 2);

        repository
            .set_favorite(FavoriteKind::Species, 25, false)
            .expect("unstarring should succeed");
        let entries = repository.favorite_entries().expect("entries should load");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].entity_id, 6);
    }

    #[test]
    fn opens_a_file_database_and_creates_its_directory() {
        let root = std::env::temp_dir().join(format!(
            "dexorium-user-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let path = root.join("nested").join("user.db");
        let _ = std::fs::remove_dir_all(&root);

        let repository = SqliteUserRepository::open(&path).expect("user database should open");
        repository
            .set_favorite(FavoriteKind::Species, 1, true)
            .expect("starring should succeed");
        assert_eq!(repository.count(FavoriteKind::Species).unwrap(), 1);
        assert!(path.is_file());
        drop(repository);

        let reopened = SqliteUserRepository::open(&path).expect("user database should reopen");
        assert_eq!(reopened.count(FavoriteKind::Species).unwrap(), 1);

        drop(reopened);
        let _ = std::fs::remove_dir_all(&root);
    }
}

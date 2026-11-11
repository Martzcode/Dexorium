//! Shared application state managed by Tauri.

use std::sync::{Mutex, MutexGuard};

use pdx_infra::SqliteSpeciesRepository;

/// State injected into every command handler.
///
/// The reference database is seeded once at startup. Later slices will open
/// the deployed `reference.db` instead of the fixture.
pub struct AppState {
    species: Mutex<SqliteSpeciesRepository>,
}

impl AppState {
    /// Builds the shared state and seeds the in-memory reference database.
    ///
    /// # Errors
    ///
    /// Returns a message when the fixture database cannot be built.
    pub fn load() -> Result<Self, String> {
        Ok(Self {
            species: Mutex::new(SqliteSpeciesRepository::from_fixture()?),
        })
    }

    /// Locks the species repository for the duration of a command.
    ///
    /// # Errors
    ///
    /// Returns a message when a previous command panicked while holding the
    /// lock.
    pub fn species(&self) -> Result<MutexGuard<'_, SqliteSpeciesRepository>, String> {
        self.species
            .lock()
            .map_err(|_| "reference database lock is poisoned".to_owned())
    }
}

//! Shared application state managed by Tauri.

use std::path::PathBuf;
use std::sync::{Mutex, MutexGuard};

use pdx_app::SpeciesRepository;
use pdx_infra::{SqliteSpeciesRepository, SqliteUserRepository, user_db_path};
use tauri::Manager;

/// File name of the generated reference database.
const REFERENCE_DB_FILE: &str = "reference.db";

/// State injected into every command handler.
///
/// The reference database is opened once at startup. The generated
/// `reference.db` (built by `pdx-dataset`) is preferred; when it is missing,
/// the bundled fixture keeps the application usable for UI work and tests.
pub struct AppState {
    species: Mutex<SqliteSpeciesRepository>,
    user: Mutex<SqliteUserRepository>,
}

impl AppState {
    /// Builds the shared state from the best available reference database.
    ///
    /// Resolution order:
    /// 1. `DEXORIUM_REFERENCE_DB` (explicit override, used by tests and CI);
    /// 2. the Tauri resource directory (packaged application);
    /// 3. `src-tauri/resources/reference.db` (local development);
    /// 4. the in-memory fixture (last resort).
    ///
    /// # Errors
    ///
    /// Returns a message when the fixture database cannot be built and no
    /// reference database is available.
    pub fn load(app: &tauri::AppHandle) -> Result<Self, String> {
        let species = match reference_repository(app) {
            Some(repository) => repository,
            None => {
                log::warn!("no usable reference.db found; falling back to the bundled fixture");
                SqliteSpeciesRepository::from_fixture()?
            }
        };

        Ok(Self {
            species: Mutex::new(species),
            user: Mutex::new(user_repository(app)),
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

    /// Locks the user database for the duration of a command.
    ///
    /// # Errors
    ///
    /// Returns a message when a previous command panicked while holding the
    /// lock.
    pub fn user(&self) -> Result<MutexGuard<'_, SqliteUserRepository>, String> {
        self.user
            .lock()
            .map_err(|_| "user database lock is poisoned".to_owned())
    }
}

/// Opens the user database, falling back to an in-memory one.
fn user_repository(app: &tauri::AppHandle) -> SqliteUserRepository {
    match app.path().app_data_dir() {
        Ok(directory) => {
            let path = user_db_path(&directory);
            match SqliteUserRepository::open(&path) {
                Ok(repository) => {
                    log::info!("user database loaded from {}", path.display());
                    return repository;
                }
                Err(error) => log::warn!(
                    "cannot open {} ({error}); favourites will not persist",
                    path.display()
                ),
            }
        }
        Err(error) => log::warn!("cannot resolve the application data directory: {error}"),
    }

    SqliteUserRepository::in_memory().expect("in-memory user database should always open")
}

/// Opens the first reference database that passes a smoke test.
fn reference_repository(app: &tauri::AppHandle) -> Option<SqliteSpeciesRepository> {
    for path in reference_candidates(app) {
        if !path.is_file() {
            continue;
        }

        match SqliteSpeciesRepository::open(&path) {
            Ok(repository) => match repository.search_species("", "en", 1) {
                Ok(_) => {
                    log::info!("reference database loaded from {}", path.display());
                    return Some(repository);
                }
                Err(error) => log::warn!(
                    "ignoring {}: the database is not readable ({error})",
                    path.display()
                ),
            },
            Err(error) => log::warn!("ignoring {}: {error}", path.display()),
        }
    }

    None
}

/// Candidate locations, most specific first.
fn reference_candidates(app: &tauri::AppHandle) -> Vec<PathBuf> {
    let mut candidates = Vec::new();

    if let Some(path) = std::env::var_os("DEXORIUM_REFERENCE_DB") {
        candidates.push(PathBuf::from(path));
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        candidates.push(resource_dir.join(REFERENCE_DB_FILE));
    }

    candidates.push(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join(REFERENCE_DB_FILE),
    );

    candidates
}

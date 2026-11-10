use std::path::{Path, PathBuf};

/// Read-only reference database shipped with the application.
pub const REFERENCE_DB_FILE: &str = "reference.db";

/// Read-write database holding favourites, teams, notes and settings.
pub const USER_DB_FILE: &str = "user.db";

/// Returns the deployed reference database path.
#[must_use]
pub fn reference_db_path(data_dir: &Path) -> PathBuf {
    data_dir.join("data").join(REFERENCE_DB_FILE)
}

/// Returns the user database path.
#[must_use]
pub fn user_db_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(USER_DB_FILE)
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{REFERENCE_DB_FILE, USER_DB_FILE, reference_db_path, user_db_path};

    #[test]
    fn keeps_reference_and_user_data_separate() {
        let root = Path::new("root");

        assert!(reference_db_path(root).ends_with(REFERENCE_DB_FILE));
        assert!(user_db_path(root).ends_with(USER_DB_FILE));
        assert_ne!(reference_db_path(root), user_db_path(root));
    }
}

//! Dataset provenance adapter (`dataset_meta`).

use pdx_app::DatasetMetadata;
use pdx_domain::DatasetRowCount;

use super::repository::{SqliteSpeciesRepository, db_error};

impl pdx_app::DatasetRepository for SqliteSpeciesRepository {
    fn dataset_metadata(&self) -> Result<DatasetMetadata, String> {
        let mut statement = self
            .connection
            .prepare("SELECT key, value FROM dataset_meta")
            .map_err(db_error)?;

        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(db_error)?;

        let mut values = std::collections::HashMap::new();
        for row in rows {
            let (key, value) = row.map_err(db_error)?;
            values.insert(key, value);
        }

        let languages = values
            .get("langs")
            .map(|langs| {
                langs
                    .split(',')
                    .map(str::trim)
                    .filter(|lang| !lang.is_empty())
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();

        Ok(DatasetMetadata {
            version: read(&values, "version"),
            built_at: read(&values, "built_at"),
            source_commit: read(&values, "source_commit"),
            source_lock: read(&values, "source_lock"),
            schema_version: read(&values, "schema_version"),
            languages,
            row_counts: parse_row_counts(values.get("row_counts_json")),
        })
    }
}

fn read(values: &std::collections::HashMap<String, String>, key: &str) -> String {
    values.get(key).cloned().unwrap_or_default()
}

fn parse_row_counts(raw: Option<&String>) -> Vec<DatasetRowCount> {
    let Some(raw) = raw else {
        return Vec::new();
    };

    let Ok(counts) = serde_json::from_str::<std::collections::HashMap<String, i64>>(raw) else {
        return Vec::new();
    };

    let mut counts: Vec<DatasetRowCount> = counts
        .into_iter()
        .map(|(table, count)| DatasetRowCount { table, count })
        .collect();
    counts.sort_by(|left, right| left.table.cmp(&right.table));
    counts
}

#[cfg(test)]
mod tests {
    use pdx_app::DatasetRepository;

    use super::SqliteSpeciesRepository;

    #[test]
    fn fixture_reports_its_provenance() {
        let repository =
            SqliteSpeciesRepository::from_fixture().expect("fixture database should build");

        let metadata = repository
            .dataset_metadata()
            .expect("metadata should load");

        assert_eq!(metadata.version, "0.1.0-fixture");
        assert_eq!(metadata.languages, Vec::<String>::new());
        assert!(
            metadata
                .row_counts
                .iter()
                .any(|entry| entry.table == "species" && entry.count == 31)
        );
    }

    #[test]
    fn generated_database_reports_the_build_metadata() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("resources")
            .join("reference.db");
        if !path.is_file() {
            return;
        }

        let repository = SqliteSpeciesRepository::open(&path).expect("database should open");
        let metadata = repository
            .dataset_metadata()
            .expect("metadata should load");

        assert!(!metadata.version.is_empty());
        assert_eq!(metadata.schema_version, "2");
        assert_eq!(metadata.source_commit.len(), 40);
        assert!(metadata.languages.contains(&"en".to_owned()));
        assert!(metadata.languages.contains(&"fr".to_owned()));
        assert!(
            metadata
                .row_counts
                .iter()
                .any(|entry| entry.table == "species" && entry.count == 1_025)
        );
        assert!(
            metadata
                .row_counts
                .windows(2)
                .all(|pair| pair[0].table <= pair[1].table)
        );
    }
}

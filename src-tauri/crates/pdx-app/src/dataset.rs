use pdx_domain::{DatasetInfo, DatasetRowCount};

/// Raw provenance metadata read from `dataset_meta`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetMetadata {
    pub version: String,
    pub built_at: String,
    pub source_commit: String,
    pub source_lock: String,
    pub schema_version: String,
    pub languages: Vec<String>,
    pub row_counts: Vec<DatasetRowCount>,
}

/// Persistence port for the reference dataset metadata.
pub trait DatasetRepository {
    fn dataset_metadata(&self) -> Result<DatasetMetadata, String>;
}

/// Upstream project the reference dataset is built from.
pub const DATASET_SOURCE: &str = "PokeAPI/api-data";

/// Upstream repository.
pub const DATASET_SOURCE_URL: &str = "https://github.com/PokeAPI/api-data";

/// License of the upstream data.
pub const DATASET_LICENSE: &str = "BSD-3-Clause";

/// Upstream license text.
pub const DATASET_LICENSE_URL: &str =
    "https://github.com/PokeAPI/api-data/blob/master/LICENSE.txt";

/// Loads dataset provenance and enriches it with the upstream attribution.
pub fn dataset_info(repository: &dyn DatasetRepository) -> Result<DatasetInfo, String> {
    let metadata = repository.dataset_metadata()?;

    Ok(DatasetInfo {
        version: metadata.version,
        built_at: metadata.built_at,
        source_commit: metadata.source_commit,
        source_lock: metadata.source_lock,
        schema_version: metadata.schema_version,
        languages: metadata.languages,
        row_counts: metadata.row_counts,
        license: DATASET_LICENSE.to_owned(),
        license_url: DATASET_LICENSE_URL.to_owned(),
        source: DATASET_SOURCE.to_owned(),
        source_url: DATASET_SOURCE_URL.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use pdx_domain::DatasetRowCount;

    use super::{
        DATASET_LICENSE, DATASET_SOURCE, DatasetMetadata, DatasetRepository, dataset_info,
    };

    struct FakeDataset;

    impl DatasetRepository for FakeDataset {
        fn dataset_metadata(&self) -> Result<DatasetMetadata, String> {
            Ok(DatasetMetadata {
                version: "2026.11.0".to_owned(),
                built_at: "2026-11-14T00:00:00Z".to_owned(),
                source_commit: "447bd5c".to_owned(),
                source_lock: "lock".to_owned(),
                schema_version: "2".to_owned(),
                languages: vec!["en".to_owned(), "fr".to_owned()],
                row_counts: vec![DatasetRowCount {
                    table: "species".to_owned(),
                    count: 1_025,
                }],
            })
        }
    }

    #[test]
    fn exposes_the_upstream_attribution() {
        let info = dataset_info(&FakeDataset).expect("metadata should load");

        assert_eq!(info.source, DATASET_SOURCE);
        assert_eq!(info.license, DATASET_LICENSE);
        assert!(info.source_url.starts_with("https://"));
        assert_eq!(info.row_counts[0].count, 1_025);
        assert_eq!(info.languages, vec!["en", "fr"]);
    }
}

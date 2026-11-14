use serde::{Deserialize, Serialize};

/// Number of rows stored for one table of the reference database.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetRowCount {
    pub table: String,
    pub count: i64,
}

/// Provenance and contents of the embedded reference dataset.
///
/// The values are read from the `dataset_meta` table written by
/// `pdx-dataset`. They let the settings page prove what the application ships
/// and where it came from, without any network access.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatasetInfo {
    pub version: String,
    pub built_at: String,
    pub source_commit: String,
    pub source_lock: String,
    pub schema_version: String,
    pub languages: Vec<String>,
    pub row_counts: Vec<DatasetRowCount>,
    pub license: String,
    pub license_url: String,
    pub source: String,
    pub source_url: String,
}

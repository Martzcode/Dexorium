use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationSummary {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub region_slug: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationAreaSummary {
    pub id: i64,
    pub slug: String,
    pub location_slug: String,
    pub game_index: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSummary {
    pub species_id: i64,
    pub species_slug: String,
    pub species_name: String,
    pub version_slug: String,
    pub location_area_slug: String,
    pub method_slug: String,
    pub min_level: i64,
    pub max_level: i64,
    pub chance: i64,
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationDetail {
    pub location: LocationSummary,
    pub areas: Vec<LocationAreaSummary>,
    pub encounters: Vec<EncounterSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NatureSummary {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub increased_stat_slug: Option<String>,
    pub decreased_stat_slug: Option<String>,
}

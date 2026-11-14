use serde::{Deserialize, Serialize};

/// Filtered, sorted and paginated ability list query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub generations: Vec<u16>,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub limit: u32,
    #[serde(default = "default_lang")]
    pub lang: String,
}

impl Default for AbilityListQuery {
    fn default() -> Self {
        Self {
            search: None,
            generations: Vec::new(),
            offset: 0,
            limit: 0,
            lang: default_lang(),
        }
    }
}

/// Compact ability row used by the catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilitySummary {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub generation_id: u16,
    pub is_main_series: bool,
    pub holder_count: u32,
}

/// One page of abilities plus the total number of matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityPage {
    pub items: Vec<AbilitySummary>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Species or form that has an ability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityHolder {
    pub form_id: i64,
    pub species_id: i64,
    pub national_dex_number: u16,
    pub species_name: String,
    pub form_name: String,
    pub slot: u8,
    pub is_hidden: bool,
}

/// Aggregated ability detail payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityDetail {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub generation_id: u16,
    pub is_main_series: bool,
    pub effect_short: Option<String>,
    pub effect_long: Option<String>,
    pub description: Option<String>,
    pub holders: Vec<AbilityHolder>,
}

fn default_lang() -> String {
    "en".to_owned()
}

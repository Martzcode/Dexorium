use serde::{Deserialize, Serialize};

use crate::SortDirection;

/// Sortable move column.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MoveSortField {
    Id,
    Name,
    Power,
    Accuracy,
    Pp,
}

/// Filtered, sorted and paginated move list query.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub types: Vec<String>,
    #[serde(default)]
    pub damage_classes: Vec<String>,
    #[serde(default)]
    pub generations: Vec<u16>,
    #[serde(default)]
    pub min_power: Option<u16>,
    #[serde(default)]
    pub max_power: Option<u16>,
    #[serde(default)]
    pub sort: Vec<MoveSortCriterion>,
    #[serde(default)]
    pub offset: u32,
    #[serde(default)]
    pub limit: u32,
    #[serde(default = "default_lang")]
    pub lang: String,
}

impl Default for MoveListQuery {
    fn default() -> Self {
        Self {
            search: None,
            types: Vec::new(),
            damage_classes: Vec::new(),
            generations: Vec::new(),
            min_power: None,
            max_power: None,
            sort: Vec::new(),
            offset: 0,
            limit: 0,
            lang: default_lang(),
        }
    }
}

/// One sort criterion for the move catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveSortCriterion {
    pub field: MoveSortField,
    pub direction: SortDirection,
}

/// Compact move row used by the catalogue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveSummary {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub type_slug: String,
    pub damage_class: String,
    pub power: Option<u16>,
    pub accuracy: Option<u16>,
    pub pp: Option<u16>,
    pub generation_id: u16,
}

/// One page of moves plus the total number of matches.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MovePage {
    pub items: Vec<MoveSummary>,
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Species or form that can learn a move in the latest version group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveLearner {
    pub form_id: i64,
    pub species_id: i64,
    pub national_dex_number: u16,
    pub species_name: String,
    pub form_name: String,
    pub methods: Vec<String>,
    pub level: Option<u16>,
}

/// Aggregated move detail payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveDetail {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub generation_id: u16,
    pub type_slug: String,
    pub damage_class: String,
    pub power: Option<u16>,
    pub accuracy: Option<u16>,
    pub pp: Option<u16>,
    pub priority: i16,
    pub effect_chance: Option<u16>,
    pub target: String,
    pub ailment: Option<String>,
    pub crit_rate_bonus: u8,
    pub drain_pct: i16,
    pub healing_pct: i16,
    pub min_hits: Option<u16>,
    pub max_hits: Option<u16>,
    pub min_turns: Option<u16>,
    pub max_turns: Option<u16>,
    pub flinch_chance: u16,
    pub stat_chance: u16,
    pub flags_bitmask: u32,
    pub effect_short: Option<String>,
    pub effect_long: Option<String>,
    pub description: Option<String>,
    /// Version group used to resolve learners.
    pub learner_version_group: Option<String>,
    pub learners: Vec<MoveLearner>,
}

/// One learnable move for a form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnsetEntry {
    pub move_id: i64,
    pub slug: String,
    pub name: String,
    pub type_slug: String,
    pub damage_class: String,
    pub power: Option<u16>,
    pub accuracy: Option<u16>,
    pub pp: Option<u16>,
    pub method: String,
    pub level: u16,
    pub machine: Option<String>,
}

/// Version group available for a learnset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionGroupRef {
    pub id: i64,
    pub slug: String,
    pub generation_id: u16,
    pub order_index: u16,
}

/// Learnset of one form for a given version group.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LearnsetPage {
    pub form_id: i64,
    pub version_group: Option<VersionGroupRef>,
    pub version_groups: Vec<VersionGroupRef>,
    pub entries: Vec<LearnsetEntry>,
    pub total: u32,
}

fn default_lang() -> String {
    "en".to_owned()
}

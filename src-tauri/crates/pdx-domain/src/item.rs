use serde::{Deserialize, Serialize};

/// Summary row shown in the item catalogue.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub category_slug: Option<String>,
    pub pocket_slug: Option<String>,
    pub cost: Option<i64>,
    pub is_berry: bool,
    pub fling_power: Option<i64>,
}

/// Berry-specific data attached to an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BerryDetail {
    pub firmness_slug: Option<String>,
    pub growth_time: Option<i64>,
    pub max_harvest: Option<i64>,
    pub natural_gift_power: Option<i64>,
    pub size: Option<i64>,
    pub smoothness: Option<i64>,
    pub soil_dryness: Option<i64>,
    pub natural_gift_type_slug: Option<String>,
    pub flavors: Vec<(String, i64)>,
}

/// Full item payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemDetail {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub category_slug: Option<String>,
    pub pocket_slug: Option<String>,
    pub cost: Option<i64>,
    pub is_berry: bool,
    pub fling_power: Option<i64>,
    pub fling_effect_slug: Option<String>,
    pub flags: Vec<String>,
    pub effect: Option<String>,
    pub short_effect: Option<String>,
    pub description: Option<String>,
    pub berry: Option<BerryDetail>,
}

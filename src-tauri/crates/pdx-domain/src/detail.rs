use serde::Serialize;

use crate::BaseStats;

/// A playable form of a species.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FormSummary {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub is_default: bool,
    pub is_mega: bool,
    pub is_gigantamax: bool,
    pub is_regional: bool,
    pub region: Option<String>,
    pub generation_id: u16,
    pub types: Vec<String>,
    pub stats: BaseStats,
    pub bst: u16,
    pub height_dm: u16,
    pub weight_hg: u16,
}

/// Ability attached to a form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AbilityRef {
    pub id: i64,
    pub slug: String,
    pub name: String,
    pub slot: u8,
    pub is_hidden: bool,
}

/// One localized Pokédex description.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FlavorTextEntry {
    pub version_id: i64,
    pub version_slug: String,
    pub generation_id: u16,
    pub text: String,
}

/// Defensive multiplier of one attacking type against the selected form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeEffectiveness {
    pub attacking_type: String,
    /// 0, 25, 50, 100, 200 or 400 (percent).
    pub multiplier_percent: u16,
}

/// Flat evolution edge; the client rebuilds the tree from `from`/`to` ids.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvolutionStepView {
    pub from_species_id: Option<i64>,
    pub from_name: Option<String>,
    pub to_species_id: i64,
    pub to_name: String,
    pub to_national_dex_number: u16,
    pub trigger: String,
    pub min_level: Option<u16>,
    pub trigger_item: Option<String>,
    pub held_item: Option<String>,
    pub time_of_day: Option<String>,
    pub location: Option<String>,
    pub known_move: Option<String>,
    pub known_move_type: Option<String>,
    pub min_happiness: Option<u16>,
    pub min_affection: Option<u16>,
    pub min_beauty: Option<u16>,
    pub gender: Option<String>,
    pub needs_overworld_rain: bool,
    pub turn_upside_down: bool,
    pub relative_physical_stats: Option<i16>,
    pub condition_expression: Option<String>,
    pub extra_condition_json: Option<String>,
    pub trade_species: Option<String>,
    pub party_species: Option<String>,
    pub party_type: Option<String>,
}

/// Aggregated payload for the species detail page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PokemonDetail {
    pub species_id: i64,
    pub national_dex_number: u16,
    pub slug: String,
    pub name: String,
    pub genus: Option<String>,
    pub generation_id: u16,
    pub is_legendary: bool,
    pub is_mythical: bool,
    pub is_baby: bool,
    pub is_ultra_beast: bool,
    pub capture_rate: u16,
    pub base_happiness: Option<u16>,
    pub hatch_counter: Option<u16>,
    /// -1 for genderless, otherwise eighths of female.
    pub gender_rate: i16,
    pub growth_rate: String,
    pub color: Option<String>,
    pub shape: Option<String>,
    pub habitat: Option<String>,
    pub egg_groups: Vec<String>,
    pub form: FormSummary,
    pub forms: Vec<FormSummary>,
    pub abilities: Vec<AbilityRef>,
    pub evolution_chain_id: Option<i64>,
    pub evolutions: Vec<EvolutionStepView>,
    pub flavor_texts: Vec<FlavorTextEntry>,
    pub effectiveness: Vec<TypeEffectiveness>,
    pub effectiveness_generation_id: u16,
}

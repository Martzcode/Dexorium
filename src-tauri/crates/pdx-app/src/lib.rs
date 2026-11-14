//! Application layer for Dexorium.
//!
//! Use cases orchestrate the pure domain and talk to the outside world through
//! ports. Tauri commands stay thin and call into this crate.

pub mod abilities;
pub mod app_info;
pub mod dataset;
pub mod favorites;
pub mod moves;
pub mod search;
pub mod species;
pub mod types;

pub use abilities::{
    AbilityRepository, DEFAULT_ABILITY_PAGE_SIZE, MAX_ABILITY_PAGE_SIZE, ability_detail,
    list_abilities,
};
pub use app_info::{AppInfo, app_info};
pub use dataset::{DatasetMetadata, DatasetRepository, dataset_info};
pub use favorites::{
    FavoriteRepository, favorite_species_ids, list_favorites, set_favorite,
};
pub use moves::{
    DEFAULT_MOVE_PAGE_SIZE, MAX_MOVE_PAGE_SIZE, MoveRepository, learnset, list_moves, move_detail,
};
pub use search::{
    DEFAULT_SEARCH_HITS, DEFAULT_SUGGESTIONS, MAX_SEARCH_HITS, SearchRepository, search_global,
    suggest,
};
pub use species::{
    DEFAULT_PAGE_SIZE, DEFAULT_SEARCH_LIMIT, LATEST_GENERATION, MAX_PAGE_SIZE, MAX_SEARCH_LIMIT,
    SpeciesRepository, list_species, pokemon_detail, search_species, species_detail,
};
pub use types::{LATEST_TYPE_GENERATION, TypeRepository, type_matrix};

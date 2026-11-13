//! Application layer for Dexorium.
//!
//! Use cases orchestrate the pure domain and talk to the outside world through
//! ports. Tauri commands stay thin and call into this crate.

pub mod app_info;
pub mod species;

pub use app_info::{AppInfo, app_info};
pub use species::{
    DEFAULT_PAGE_SIZE, DEFAULT_SEARCH_LIMIT, LATEST_GENERATION, MAX_PAGE_SIZE, MAX_SEARCH_LIMIT,
    SpeciesRepository, list_species, pokemon_detail, search_species, species_detail,
};

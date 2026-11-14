//! Pure domain layer for Dexorium.
//!
//! This crate owns entities, value objects and business rules. It must never
//! depend on Tauri, SQLite, the file system or any other I/O crate.

pub mod ability;
pub mod dataset;
pub mod detail;
pub mod favorites;
pub mod moves;
pub mod search;
pub mod species;
pub mod types;

pub use ability::{
    AbilityDetail, AbilityHolder, AbilityListQuery, AbilityPage, AbilitySummary,
};
pub use dataset::{DatasetInfo, DatasetRowCount};
pub use detail::{
    AbilityRef, EvolutionStepView, FlavorTextEntry, FormSummary, PokemonDetail, TypeEffectiveness,
};
pub use favorites::{FavoriteCollection, FavoriteEntry, FavoriteKind, FavoriteStatus};
pub use moves::{
    LearnsetEntry, LearnsetPage, MoveDetail, MoveLearner, MoveListQuery, MovePage, MoveSortCriterion,
    MoveSortField, MoveSummary, VersionGroupRef,
};
pub use search::{
    GlobalSearchQuery, GlobalSearchResult, SearchEntityType, SearchHit,
};
pub use species::{
    BaseStats, NationalDexNumber, SortCriterion, SortDirection, SpeciesDetail, SpeciesFlags,
    SpeciesListQuery, SpeciesPage, SpeciesSortField, SpeciesStat, SpeciesSummary, StatRange,
    TypeMatchMode,
};
pub use types::{TypeMatrix, TypeRef};

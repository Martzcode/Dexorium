//! Pure domain layer for Dexorium.
//!
//! This crate owns entities, value objects and business rules. It must never
//! depend on Tauri, SQLite, the file system or any other I/O crate.

pub mod detail;
pub mod species;

pub use detail::{
    AbilityRef, EvolutionStepView, FlavorTextEntry, FormSummary, PokemonDetail, TypeEffectiveness,
};
pub use species::{
    BaseStats, NationalDexNumber, SortCriterion, SortDirection, SpeciesDetail, SpeciesFlags,
    SpeciesListQuery, SpeciesPage, SpeciesSortField, SpeciesStat, SpeciesSummary, StatRange,
    TypeMatchMode,
};

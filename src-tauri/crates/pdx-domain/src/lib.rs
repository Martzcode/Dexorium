//! Pure domain layer for Dexorium.
//!
//! This crate owns entities, value objects and business rules. It must never
//! depend on Tauri, SQLite, the file system or any other I/O crate.

pub mod species;

pub use species::{BaseStats, NationalDexNumber, SpeciesDetail, SpeciesSummary};

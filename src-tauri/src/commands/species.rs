use pdx_domain::{SpeciesDetail, SpeciesListQuery, SpeciesPage, SpeciesSummary};
use tauri::State;

use crate::state::AppState;

/// Returns one filtered, sorted and paginated page of species.
#[tauri::command]
pub fn list_pokemon(
    state: State<'_, AppState>,
    query: SpeciesListQuery,
) -> Result<SpeciesPage, String> {
    let repository = state.species()?;
    pdx_app::list_species(&*repository, &query)
}

/// Searches species by localized name, English name, slug or national number.
#[tauri::command]
pub fn search_species(
    state: State<'_, AppState>,
    query: String,
    lang: String,
    limit: Option<usize>,
) -> Result<Vec<SpeciesSummary>, String> {
    let repository = state.species()?;
    pdx_app::search_species(&*repository, &query, &lang, limit)
}

/// Returns the detail payload for one species, or `null` when absent.
#[tauri::command]
pub fn get_species(
    state: State<'_, AppState>,
    id: i64,
    lang: String,
) -> Result<Option<SpeciesDetail>, String> {
    let repository = state.species()?;
    pdx_app::species_detail(&*repository, id, &lang)
}

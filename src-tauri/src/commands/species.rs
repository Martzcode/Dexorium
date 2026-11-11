use pdx_domain::{SpeciesDetail, SpeciesSummary};
use tauri::State;

use crate::state::AppState;

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

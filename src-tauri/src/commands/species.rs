use pdx_domain::{PokemonDetail, SpeciesDetail, SpeciesListQuery, SpeciesPage, SpeciesSummary};
use tauri::State;

use crate::state::AppState;

/// Returns one filtered, sorted and paginated page of species.
#[tauri::command]
pub fn list_pokemon(
    state: State<'_, AppState>,
    mut query: SpeciesListQuery,
) -> Result<SpeciesPage, String> {
    if query.favorites_only {
        let user = state.user()?;
        query.favorite_ids = Some(pdx_app::favorite_species_ids(&*user)?);
    }

    let repository = state.species()?;
    pdx_app::list_species(&*repository, &query)
}

/// Returns the aggregated detail payload for one species and one of its forms.
#[tauri::command]
pub fn get_pokemon_detail(
    state: State<'_, AppState>,
    species_id: i64,
    form_id: Option<i64>,
    lang: String,
    generation: Option<u16>,
) -> Result<Option<PokemonDetail>, String> {
    let repository = state.species()?;
    pdx_app::pokemon_detail(&*repository, species_id, form_id, &lang, generation)
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

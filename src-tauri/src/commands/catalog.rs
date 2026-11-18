use pdx_domain::{
    AbilityDetail, AbilityListQuery, AbilityPage, EncounterSummary, GlobalSearchQuery,
    GlobalSearchResult, ItemDetail, ItemSummary, LearnsetPage, LocationDetail, LocationSummary,
    MoveDetail, MoveListQuery, MovePage, NatureSummary,
};
use tauri::State;

use crate::state::AppState;

/// Returns one filtered, sorted and paginated page of moves.
#[tauri::command]
pub fn list_moves(state: State<'_, AppState>, query: MoveListQuery) -> Result<MovePage, String> {
    let repository = state.species()?;
    pdx_app::list_moves(&*repository, &query)
}

/// Returns one move with its localized effect and its learners.
#[tauri::command]
pub fn get_move_detail(
    state: State<'_, AppState>,
    id: i64,
    lang: String,
) -> Result<Option<MoveDetail>, String> {
    let repository = state.species()?;
    pdx_app::move_detail(&*repository, id, &lang)
}

/// Returns the learnset of one form, optionally for a given version group.
#[tauri::command]
pub fn get_pokemon_learnset(
    state: State<'_, AppState>,
    form_id: i64,
    lang: String,
    version_group: Option<String>,
) -> Result<LearnsetPage, String> {
    let repository = state.species()?;
    pdx_app::learnset(&*repository, form_id, &lang, version_group.as_deref())
}

/// Returns one filtered, sorted and paginated page of abilities.
#[tauri::command]
pub fn list_abilities(
    state: State<'_, AppState>,
    query: AbilityListQuery,
) -> Result<AbilityPage, String> {
    let repository = state.species()?;
    pdx_app::list_abilities(&*repository, &query)
}

/// Returns one ability with its localized effect and its holders.
#[tauri::command]
pub fn get_ability_detail(
    state: State<'_, AppState>,
    id: i64,
    lang: String,
) -> Result<Option<AbilityDetail>, String> {
    let repository = state.species()?;
    pdx_app::ability_detail(&*repository, id, &lang)
}

/// Searches species, moves and abilities in one call.
#[tauri::command]
pub fn search_global(
    state: State<'_, AppState>,
    query: GlobalSearchQuery,
) -> Result<GlobalSearchResult, String> {
    let repository = state.species()?;
    pdx_app::search_global(&*repository, &query)
}

/// Lists items with optional search and category filter.
#[tauri::command]
pub fn list_items(
    state: State<'_, AppState>,
    search: Option<String>,
    category: Option<String>,
    lang: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<ItemSummary>, String> {
    let repository = state.species()?;
    pdx_app::list_items(
        &*repository,
        search.as_deref(),
        category.as_deref(),
        &lang,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    )
}

/// Returns one item with its localized effect and berry data.
#[tauri::command]
pub fn get_item_detail(
    state: State<'_, AppState>,
    id: i64,
    lang: String,
) -> Result<Option<ItemDetail>, String> {
    let repository = state.species()?;
    pdx_app::item_detail(&*repository, id, &lang)
}

/// Lists locations with optional search.
#[tauri::command]
pub fn list_locations(
    state: State<'_, AppState>,
    search: Option<String>,
    lang: String,
    limit: Option<u32>,
    offset: Option<u32>,
) -> Result<Vec<LocationSummary>, String> {
    let repository = state.species()?;
    pdx_app::list_locations(
        &*repository,
        search.as_deref(),
        &lang,
        limit.unwrap_or(100),
        offset.unwrap_or(0),
    )
}

/// Returns one location with its areas and encounters.
#[tauri::command]
pub fn get_location_detail(
    state: State<'_, AppState>,
    id: i64,
    lang: String,
) -> Result<Option<LocationDetail>, String> {
    let repository = state.species()?;
    pdx_app::location_detail(&*repository, id, &lang)
}

/// Returns encounters for one species ("where to catch").
#[tauri::command]
pub fn get_species_encounters(
    state: State<'_, AppState>,
    species_id: i64,
    lang: String,
) -> Result<Vec<EncounterSummary>, String> {
    let repository = state.species()?;
    pdx_app::encounters_for_species(&*repository, species_id, &lang)
}

/// Lists natures.
#[tauri::command]
pub fn list_natures(
    state: State<'_, AppState>,
    lang: String,
) -> Result<Vec<NatureSummary>, String> {
    let repository = state.species()?;
    pdx_app::list_natures(&*repository, &lang)
}


/// Returns a short list of suggestions for the command palette.
#[tauri::command]
pub fn suggest(
    state: tauri::State<'_, crate::state::AppState>,
    query: String,
    lang: String,
    limit: Option<usize>,
) -> Result<Vec<pdx_domain::SearchHit>, String> {
    let repository = state.species()?;
    pdx_app::suggest(&*repository, &query, &lang, limit)
}

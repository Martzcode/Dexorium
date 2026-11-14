use pdx_domain::{
    AbilityDetail, AbilityListQuery, AbilityPage, GlobalSearchQuery, GlobalSearchResult,
    LearnsetPage, MoveDetail, MoveListQuery, MovePage, SearchHit,
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

/// Returns a short list of suggestions for the command palette.
#[tauri::command]
pub fn suggest(
    state: State<'_, AppState>,
    query: String,
    lang: String,
    limit: Option<usize>,
) -> Result<Vec<SearchHit>, String> {
    let repository = state.species()?;
    pdx_app::suggest(&*repository, &query, &lang, limit)
}

use pdx_domain::{FavoriteCollection, FavoriteKind};
use tauri::State;

use crate::state::AppState;

/// Returns the user's favourite species, newest first.
#[tauri::command]
pub fn list_favorites(state: State<'_, AppState>) -> Result<FavoriteCollection, String> {
    let repository = state.user()?;
    pdx_app::list_favorites(&*repository)
}

/// Stars or unstars a species and returns the updated collection.
#[tauri::command]
pub fn set_favorite(
    state: State<'_, AppState>,
    species_id: i64,
    favorite: bool,
) -> Result<FavoriteCollection, String> {
    let repository = state.user()?;
    pdx_app::set_favorite(
        &*repository,
        FavoriteKind::Species,
        species_id,
        favorite,
    )
}

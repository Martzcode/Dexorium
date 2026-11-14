use pdx_domain::TypeMatrix;
use tauri::State;

use crate::state::AppState;

/// Returns the type effectiveness chart for one generation.
#[tauri::command]
pub fn get_type_matrix(
    state: State<'_, AppState>,
    generation: Option<u8>,
    lang: String,
) -> Result<TypeMatrix, String> {
    let repository = state.species()?;
    pdx_app::type_matrix(&*repository, generation, &lang)
}

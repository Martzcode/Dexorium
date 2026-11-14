use pdx_domain::DatasetInfo;
use tauri::State;

use crate::state::AppState;

/// Returns the provenance and contents of the embedded reference dataset.
#[tauri::command]
pub fn get_dataset_info(state: State<'_, AppState>) -> Result<DatasetInfo, String> {
    let repository = state.species()?;
    pdx_app::dataset_info(&*repository)
}

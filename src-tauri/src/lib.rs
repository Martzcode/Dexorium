mod commands;
mod state;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::catalog::list_moves,
            commands::catalog::get_move_detail,
            commands::catalog::get_pokemon_learnset,
            commands::catalog::list_abilities,
            commands::catalog::get_ability_detail,
            commands::catalog::search_global,
            commands::catalog::suggest,
            commands::collection::list_favorites,
            commands::collection::set_favorite,
            commands::dataset::get_dataset_info,
            commands::species::list_pokemon,
            commands::species::get_pokemon_detail,
            commands::species::search_species,
            commands::species::get_species,
            commands::tools::get_type_matrix,
        ])
        .setup(|app| {
            let state = AppState::load(app.handle())
                .map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
            app.manage(state);

            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            log::info!("Dexorium started");
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while building the Dexorium desktop application");
}

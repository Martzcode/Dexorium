mod commands;
mod state;

use tauri::Manager;

use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::app_info::get_app_info,
            commands::species::search_species,
            commands::species::get_species,
        ])
        .setup(|app| {
            let state =
                AppState::load().map_err(|error| -> Box<dyn std::error::Error> { error.into() })?;
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

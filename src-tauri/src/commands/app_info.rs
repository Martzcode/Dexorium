use pdx_app::AppInfo;

/// Returns build and runtime information for the Angular shell.
#[tauri::command]
pub fn get_app_info() -> AppInfo {
    pdx_app::app_info()
}

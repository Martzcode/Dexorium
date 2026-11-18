use crate::state::AppState;
use std::process::Command;
use tauri::State;

/// Runs a read-only SQL query against the reference database.
#[tauri::command]
pub fn run_sql_query(
    state: State<'_, AppState>,
    sql: String,
) -> Result<(Vec<String>, Vec<Vec<String>>), String> {
    let repository = state.species()?;
    repository.query_readonly(&sql)
}
const ALLOWED_URLS: [&str; 1] = ["https://github.com/Martzcode"];

/// Opens an allow-listed URL in the user's default browser.
///
/// Dexorium never opens arbitrary links from dataset content: only the URLs
/// compiled into this module can reach the operating system.
#[tauri::command]
pub fn open_external(url: String) -> Result<(), String> {
    if !is_allowed(&url) {
        return Err(format!("refusing to open untrusted URL: {url}"));
    }

    open_in_browser(&url).map_err(|error| format!("unable to open {url}: {error}"))
}

fn is_allowed(url: &str) -> bool {
    ALLOWED_URLS.contains(&url)
}

#[cfg(target_os = "windows")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    Command::new("rundll32.exe")
        .args(["url.dll,FileProtocolHandler", url])
        .spawn()
        .map(|_| ())
}

#[cfg(target_os = "macos")]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    Command::new("open").arg(url).spawn().map(|_| ())
}

#[cfg(all(unix, not(target_os = "macos")))]
fn open_in_browser(url: &str) -> std::io::Result<()> {
    Command::new("xdg-open").arg(url).spawn().map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::is_allowed;

    #[test]
    fn only_allow_listed_urls_can_be_opened() {
        assert!(is_allowed("https://github.com/Martzcode"));
        assert!(!is_allowed("https://github.com/Martzcode/Dexorium"));
        assert!(!is_allowed("https://example.com"));
        assert!(!is_allowed("file:///C:/Windows/System32/calc.exe"));
    }
}

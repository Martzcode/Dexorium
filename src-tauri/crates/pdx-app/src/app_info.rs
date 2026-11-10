use serde::Serialize;

/// Human-readable application name.
pub const APP_NAME: &str = "Dexorium";

/// Runtime diagnostics exposed to the Angular shell.
///
/// This is deliberately small: it proves the IPC bridge works and gives the UI
/// the information it needs to display the current runtime.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub name: String,
    pub version: String,
    pub runtime: String,
    pub platform: String,
    pub arch: String,
    pub offline: bool,
}

/// Returns build and runtime information about the desktop application.
#[must_use]
pub fn app_info() -> AppInfo {
    AppInfo {
        name: APP_NAME.to_owned(),
        version: env!("CARGO_PKG_VERSION").to_owned(),
        runtime: "tauri".to_owned(),
        platform: std::env::consts::OS.to_owned(),
        arch: std::env::consts::ARCH.to_owned(),
        offline: true,
    }
}

#[cfg(test)]
mod tests {
    use super::{APP_NAME, app_info};

    #[test]
    fn reports_the_product_identity() {
        let info = app_info();

        assert_eq!(info.name, APP_NAME);
        assert!(info.offline);
        assert_eq!(info.runtime, "tauri");
    }
}

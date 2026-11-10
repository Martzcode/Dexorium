//! Application layer for Dexorium.
//!
//! Use cases orchestrate the pure domain and talk to the outside world through
//! ports. Tauri commands stay thin and call into this crate.

pub mod app_info;

pub use app_info::{AppInfo, app_info};

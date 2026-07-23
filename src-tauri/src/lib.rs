//! Monitoring Systeme — Tauri backend entrypoint.

mod cache;
mod commands;
mod error;
mod models;
mod provider;
mod system;

use std::sync::Arc;

use cache::ProviderState;
use commands::AppState;
use provider::create_provider;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = Arc::new(ProviderState::new());
    let provider = create_provider(Arc::clone(&state));

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState { provider })
        .invoke_handler(tauri::generate_handler![
            commands::get_cpu,
            commands::get_memory,
            commands::get_disk,
            commands::get_energy,
            commands::get_battery,
            commands::get_temperature,
            commands::get_system,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

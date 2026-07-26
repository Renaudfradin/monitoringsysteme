//! Monitoring Systeme — Tauri backend entrypoint.

mod cache;
mod commands;
mod error;
mod export;
mod models;
mod plugins;
mod provider;
mod settings;
mod system;
mod tray;

use std::sync::Arc;

use cache::ProviderState;
use commands::AppState;
use plugins::PluginRegistry;
use provider::create_provider;
use settings::SettingsStore;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let provider_state = Arc::new(ProviderState::new());
    let provider = create_provider(Arc::clone(&provider_state));
    let settings = Arc::new(SettingsStore::new());
    let plugins = Arc::new(PluginRegistry::with_builtins());

    let app_state = Arc::new(AppState {
        provider,
        provider_state: Arc::clone(&provider_state),
        settings: Arc::clone(&settings),
        plugins,
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .manage(Arc::clone(&app_state))
        .setup(move |app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| std::env::temp_dir().join("monitoringsysteme"));
            let _ = std::fs::create_dir_all(&data_dir);

            {
                let mut history = provider_state.energy_history.lock();
                history.set_persist_path(data_dir.join("energy_history.json"));
                history.load_from_disk();
            }
            settings.init(data_dir.join("settings.json"));

            if let Err(e) = tray::setup_tray(app.handle()) {
                eprintln!("tray setup failed: {e}");
            }
            tray::spawn_background_sampler(app.handle().clone(), Arc::clone(&app_state));

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Keep running in the menu bar on close.
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_cpu,
            commands::get_memory,
            commands::get_disk,
            commands::get_energy,
            commands::get_battery,
            commands::get_temperature,
            commands::get_system,
            commands::get_processes,
            commands::get_network,
            commands::get_gpu,
            commands::get_energy_history_24h,
            commands::export_metrics,
            commands::check_resource_alerts,
            commands::get_settings,
            commands::set_settings,
            commands::list_plugins,
            commands::invoke_plugin,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                if let Some(state) = app.try_state::<Arc<AppState>>() {
                    state.provider_state.energy_history.lock().flush_and_save();
                }
            }
        });
}

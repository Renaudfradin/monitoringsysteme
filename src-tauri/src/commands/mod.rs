//! Tauri commands — one independent command per metric family.

use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tauri::{AppHandle, State};
use tauri_plugin_notification::NotificationExt;

use crate::error::MetricError;
use crate::export::{self, ExportSnapshot};
use crate::models::{
    AppSettings, BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, EnergySample,
    GpuLiveMetrics, MemoryMetrics, NetworkMetrics, PluginInfo, ProcessMetrics, SystemInfo,
    TemperatureMetrics,
};
use crate::plugins::PluginRegistry;
use crate::provider::SystemProvider;
use crate::settings::SettingsStore;
use crate::cache::ProviderState;

pub struct AppState {
    pub provider: Arc<dyn SystemProvider>,
    pub provider_state: Arc<ProviderState>,
    pub settings: Arc<SettingsStore>,
    pub plugins: Arc<PluginRegistry>,
}

#[tauri::command]
pub async fn get_cpu(state: State<'_, Arc<AppState>>) -> Result<CpuMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.cpu())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_memory(state: State<'_, Arc<AppState>>) -> Result<MemoryMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.memory())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_disk(state: State<'_, Arc<AppState>>) -> Result<DiskMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.disk())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_energy(state: State<'_, Arc<AppState>>) -> Result<EnergyMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.energy())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_battery(
    state: State<'_, Arc<AppState>>,
) -> Result<Option<BatteryMetrics>, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.battery())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_temperature(
    state: State<'_, Arc<AppState>>,
) -> Result<TemperatureMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.temperature())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_system(state: State<'_, Arc<AppState>>) -> Result<SystemInfo, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.system())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_processes(state: State<'_, Arc<AppState>>) -> Result<ProcessMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.processes())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_network(state: State<'_, Arc<AppState>>) -> Result<NetworkMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.network())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_gpu(state: State<'_, Arc<AppState>>) -> Result<GpuLiveMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.gpu())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_energy_history_24h(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<EnergySample>, MetricError> {
    let ps = Arc::clone(&state.provider_state);
    tauri::async_runtime::spawn_blocking(move || Ok(ps.energy_history.lock().snapshot_24h()))
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn export_metrics(
    state: State<'_, Arc<AppState>>,
    format: String,
    path: String,
) -> Result<(), MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || {
        let snapshot = ExportSnapshot {
            exported_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0),
            cpu: provider.cpu().ok(),
            memory: provider.memory().ok(),
            disk: provider.disk().ok(),
            energy: provider.energy().ok(),
            battery: provider.battery().ok().flatten(),
            temperature: provider.temperature().ok(),
            processes: provider.processes().ok(),
            network: provider.network().ok(),
            gpu: provider.gpu().ok(),
            system: provider.system().ok(),
        };
        let content = match format.to_lowercase().as_str() {
            "csv" => export::to_csv(&snapshot)?,
            _ => export::to_json(&snapshot)?,
        };
        std::fs::write(&path, content).map_err(|e| MetricError::Internal(e.to_string()))
    })
    .await
    .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn check_resource_alerts(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<String>, MetricError> {
    let settings = state.settings.get();
    if !settings.notifications_enabled {
        return Ok(vec![]);
    }

    let provider = Arc::clone(&state.provider);
    let provider_state = Arc::clone(&state.provider_state);
    let cpu_th = settings.cpu_alert_threshold;
    let ram_th = settings.ram_alert_threshold;

    let fired = tauri::async_runtime::spawn_blocking(move || {
        let mut fired = Vec::new();
        let cooldown = Duration::from_secs(5 * 60);

        if let Ok(cpu) = provider.cpu() {
            if cpu.usage >= cpu_th && provider_state.alerts.should_alert_cpu(cooldown) {
                fired.push(format!("CPU à {:.0} %", cpu.usage));
            }
        }
        if let Ok(mem) = provider.memory() {
            if mem.percent >= ram_th && provider_state.alerts.should_alert_ram(cooldown) {
                fired.push(format!("RAM à {:.0} %", mem.percent));
            }
        }
        fired
    })
    .await
    .map_err(|e| MetricError::Internal(e.to_string()))?;

    for msg in &fired {
        let _ = app
            .notification()
            .builder()
            .title("Monitoring Systeme")
            .body(msg)
            .show();
    }

    Ok(fired)
}

#[tauri::command]
pub fn get_settings(state: State<'_, Arc<AppState>>) -> Result<AppSettings, MetricError> {
    Ok(state.settings.get())
}

#[tauri::command]
pub fn set_settings(
    app: AppHandle,
    state: State<'_, Arc<AppState>>,
    settings: AppSettings,
) -> Result<AppSettings, MetricError> {
    let saved = state.settings.set(settings)?;
    crate::tray::update_tray_title(&app, state.inner());
    Ok(saved)
}

#[tauri::command]
pub fn list_plugins(state: State<'_, Arc<AppState>>) -> Result<Vec<PluginInfo>, MetricError> {
    Ok(state.plugins.list())
}

#[tauri::command]
pub async fn invoke_plugin(
    state: State<'_, Arc<AppState>>,
    id: String,
) -> Result<serde_json::Value, MetricError> {
    let provider = Arc::clone(&state.provider);
    let plugins = Arc::clone(&state.plugins);
    tauri::async_runtime::spawn_blocking(move || plugins.invoke(&id, provider.as_ref()))
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

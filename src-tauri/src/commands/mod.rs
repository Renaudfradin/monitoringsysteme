//! Tauri commands — one independent command per metric family.

use std::sync::Arc;

use tauri::State;

use crate::error::MetricError;
use crate::models::{
    BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, MemoryMetrics, SystemInfo,
    TemperatureMetrics,
};
use crate::provider::SystemProvider;

pub struct AppState {
    pub provider: Arc<dyn SystemProvider>,
}

#[tauri::command]
pub async fn get_cpu(state: State<'_, AppState>) -> Result<CpuMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.cpu())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_memory(state: State<'_, AppState>) -> Result<MemoryMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.memory())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_disk(state: State<'_, AppState>) -> Result<DiskMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.disk())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_energy(state: State<'_, AppState>) -> Result<EnergyMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.energy())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_battery(
    state: State<'_, AppState>,
) -> Result<Option<BatteryMetrics>, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.battery())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_temperature(
    state: State<'_, AppState>,
) -> Result<TemperatureMetrics, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.temperature())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

#[tauri::command]
pub async fn get_system(state: State<'_, AppState>) -> Result<SystemInfo, MetricError> {
    let provider = Arc::clone(&state.provider);
    tauri::async_runtime::spawn_blocking(move || provider.system())
        .await
        .map_err(|e| MetricError::Internal(e.to_string()))?
}

//! Shared serializable metric models returned by Tauri commands.

use serde::Serialize;

/// CPU utilisation snapshot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuMetrics {
    /// Global CPU usage percentage (0–100).
    pub usage: f32,
    /// Per-core usage percentages.
    pub cores: Vec<f32>,
    /// Current average frequency in MHz (best effort).
    pub frequency_mhz: u64,
    /// Logical core count.
    pub core_count: usize,
    /// 1-minute load average.
    pub load_avg: f64,
}

/// Memory utilisation snapshot.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    /// used / total * 100
    pub percent: f32,
}

/// Root disk volume utilisation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiskMetrics {
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub free_bytes: u64,
    pub percent: f32,
    pub name: String,
}

/// Battery state when a battery is present.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatteryMetrics {
    /// Charge level 0–100.
    pub level: f32,
    /// Human-readable state (charging, discharging, full, unknown).
    pub state: String,
    /// Remaining time in seconds, if known.
    pub time_remaining_secs: Option<u64>,
    pub charging: bool,
}

/// Single energy history sample.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnergySample {
    /// Unix timestamp (seconds).
    pub ts: u64,
    /// Instantaneous power in watts.
    pub watts: f64,
}

/// Instantaneous energy / power estimate.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnergyMetrics {
    /// Estimated instantaneous power draw (W).
    pub watts: f64,
    /// Relative to estimated machine TDP (0–100).
    pub percent: f32,
    /// Circular history for the last ~5 minutes (1 Hz).
    pub history: Vec<EnergySample>,
    /// Whether the value is estimated vs measured.
    pub estimated: bool,
}

/// Optional thermal / fan readings.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TemperatureMetrics {
    pub cpu_celsius: Option<f32>,
    pub gpu_celsius: Option<f32>,
    pub ssd_celsius: Option<f32>,
    /// Fan speeds in RPM when available.
    pub fans_rpm: Vec<f32>,
    pub available: bool,
}

/// Static / slowly changing system identity.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub arch: String,
    pub uptime_secs: u64,
    pub model: String,
}

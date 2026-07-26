//! Shared serializable metric models returned by Tauri commands.

use serde::{Deserialize, Serialize};

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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Downsampled persistent history (~1 sample / min, 24 h).
    pub history_24h: Vec<EnergySample>,
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

/// Single process entry for top lists.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessEntry {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

/// Top processes by CPU and memory.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessMetrics {
    pub top_cpu: Vec<ProcessEntry>,
    pub top_memory: Vec<ProcessEntry>,
}

/// Per-interface network rates.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkInterface {
    pub name: String,
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
    pub rx_total_bytes: u64,
    pub tx_total_bytes: u64,
}

/// Aggregate network throughput.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkMetrics {
    pub interfaces: Vec<NetworkInterface>,
    pub total_rx_bytes_per_sec: f64,
    pub total_tx_bytes_per_sec: f64,
}

/// Named fan reading.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FanInfo {
    pub name: String,
    pub rpm: f32,
}

/// Live GPU + fans (best effort).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuLiveMetrics {
    pub name: String,
    pub utilization_percent: Option<f32>,
    pub temperature_celsius: Option<f32>,
    pub fans: Vec<FanInfo>,
    pub available: bool,
}

/// User preferences persisted on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// "system" | "light" | "dark"
    pub theme: String,
    pub notifications_enabled: bool,
    pub cpu_alert_threshold: f32,
    pub ram_alert_threshold: f32,
    pub launch_at_login: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "system".into(),
            notifications_enabled: true,
            cpu_alert_threshold: 90.0,
            ram_alert_threshold: 90.0,
            launch_at_login: false,
        }
    }
}

/// Registered metric plugin descriptor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PluginInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub builtin: bool,
}

/// CPU / SoC identity and topology.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuInfo {
    /// Marketing / brand string (e.g. "Apple M3", "Intel Core i7-12700H").
    pub brand: String,
    /// Vendor when known (Apple, Intel, AMD, …).
    pub vendor: Option<String>,
    /// Logical / reported core count.
    pub cores: Option<u32>,
    pub performance_cores: Option<u32>,
    pub efficiency_cores: Option<u32>,
    /// Base / reported frequency in MHz when known.
    pub frequency_mhz: Option<u64>,
}

/// GPU / graphics adapter.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GpuInfo {
    pub name: String,
    pub chipset: Option<String>,
    pub vendor: Option<String>,
    pub cores: Option<u32>,
    pub vram_bytes: Option<u64>,
    pub metal_support: Option<String>,
    pub bus: Option<String>,
}

/// Installed memory summary (+ optional DIMM modules).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub total_bytes: Option<u64>,
    pub type_name: Option<String>,
    pub manufacturer: Option<String>,
    pub modules: Vec<MemoryModule>,
}

/// Individual RAM module / DIMM when the OS exposes it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryModule {
    pub size_bytes: Option<u64>,
    pub type_name: Option<String>,
    pub speed_mhz: Option<u32>,
    pub manufacturer: Option<String>,
    pub part_number: Option<String>,
    pub serial: Option<String>,
    pub slot: Option<String>,
}

/// Physical / logical storage device reference.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StorageInfo {
    /// Volume or friendly name.
    pub name: String,
    /// Drive model / device name (e.g. "APPLE SSD AP0256Z").
    pub model: Option<String>,
    pub medium_type: Option<String>,
    pub protocol: Option<String>,
    pub size_bytes: Option<u64>,
    pub serial: Option<String>,
    pub smart_status: Option<String>,
    pub mount_point: Option<String>,
    pub bsd_name: Option<String>,
}

/// Attached display.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DisplayInfo {
    pub name: String,
    pub resolution: Option<String>,
    pub pixel_resolution: Option<String>,
    pub display_type: Option<String>,
    pub connection: Option<String>,
    pub vendor_id: Option<String>,
    pub product_id: Option<String>,
    pub serial: Option<String>,
    pub main: bool,
}

/// Static / slowly changing system identity + hardware inventory.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemInfo {
    pub hostname: String,
    /// Machine identifier (e.g. Mac15,12) or fallback label.
    pub model: String,
    /// Marketing name (e.g. MacBook Air).
    pub model_name: Option<String>,
    /// Apple / OEM model number (e.g. MRXV3FN/A).
    pub model_number: Option<String>,
    pub serial_number: Option<String>,
    pub hardware_uuid: Option<String>,
    pub firmware_version: Option<String>,

    /// Family name: macOS, Windows, Linux, …
    pub os_name: String,
    /// User-facing version (e.g. 26.3.1, 11, Ubuntu 24.04).
    pub os_version: String,
    /// Build / edition string when available.
    pub os_build: Option<String>,
    /// Full long OS description from the platform.
    pub os_long_name: Option<String>,
    pub kernel_version: Option<String>,
    pub arch: String,
    pub uptime_secs: u64,

    pub cpu: Option<CpuInfo>,
    pub gpu: Vec<GpuInfo>,
    pub memory: Option<MemoryInfo>,
    pub storage: Vec<StorageInfo>,
    pub displays: Vec<DisplayInfo>,
}

//! Cross-platform system metric provider abstraction.
//!
//! Implement [`SystemProvider`] per OS. The factory [`create_provider`] selects
//! the correct implementation at compile time so Linux/Windows can be added
//! without changing commands or the frontend.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

use std::sync::Arc;

use crate::cache::ProviderState;
use crate::error::MetricError;
use crate::models::{
    BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, MemoryMetrics, SystemInfo,
    TemperatureMetrics,
};

/// Abstraction over OS-specific metric collection.
pub trait SystemProvider: Send + Sync {
    fn cpu(&self) -> Result<CpuMetrics, MetricError>;
    fn memory(&self) -> Result<MemoryMetrics, MetricError>;
    fn disk(&self) -> Result<DiskMetrics, MetricError>;
    fn battery(&self) -> Result<Option<BatteryMetrics>, MetricError>;
    fn energy(&self) -> Result<EnergyMetrics, MetricError>;
    fn temperature(&self) -> Result<TemperatureMetrics, MetricError>;
    fn system(&self) -> Result<SystemInfo, MetricError>;
}

/// Build the provider for the current target OS.
pub fn create_provider(state: Arc<ProviderState>) -> Arc<dyn SystemProvider> {
    #[cfg(target_os = "macos")]
    {
        return Arc::new(macos::MacOSProvider::new(state));
    }
    #[cfg(target_os = "linux")]
    {
        return Arc::new(linux::LinuxProvider::new(state));
    }
    #[cfg(target_os = "windows")]
    {
        return Arc::new(windows::WindowsProvider::new(state));
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        let _ = state;
        compile_error!("Unsupported target OS for SystemProvider");
    }
}

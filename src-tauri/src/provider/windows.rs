//! Windows provider stub — implement without touching commands or UI.
//!
//! Suggested sources: PDH, WMI, `sysinfo`, Windows Performance Counters.

use std::sync::Arc;

use super::SystemProvider;
use crate::cache::ProviderState;
use crate::error::MetricError;
use crate::models::{
    BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, MemoryMetrics, SystemInfo,
    TemperatureMetrics,
};

pub struct WindowsProvider {
    #[allow(dead_code)]
    state: Arc<ProviderState>,
}

impl WindowsProvider {
    pub fn new(state: Arc<ProviderState>) -> Self {
        Self { state }
    }
}

impl SystemProvider for WindowsProvider {
    fn cpu(&self) -> Result<CpuMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "WindowsProvider not implemented yet".into(),
        ))
    }

    fn memory(&self) -> Result<MemoryMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "WindowsProvider not implemented yet".into(),
        ))
    }

    fn disk(&self) -> Result<DiskMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "WindowsProvider not implemented yet".into(),
        ))
    }

    fn battery(&self) -> Result<Option<BatteryMetrics>, MetricError> {
        Err(MetricError::Unsupported(
            "WindowsProvider not implemented yet".into(),
        ))
    }

    fn energy(&self) -> Result<EnergyMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "WindowsProvider not implemented yet".into(),
        ))
    }

    fn temperature(&self) -> Result<TemperatureMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "WindowsProvider not implemented yet".into(),
        ))
    }

    fn system(&self) -> Result<SystemInfo, MetricError> {
        Err(MetricError::Unsupported(
            "WindowsProvider not implemented yet".into(),
        ))
    }
}

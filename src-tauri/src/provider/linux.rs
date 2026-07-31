//! Linux provider stub — implement without touching commands or UI.
//!
//! Suggested sources: `/proc`, `/sys`, `sysinfo`, RAPL for energy.

use std::sync::Arc;

use super::SystemProvider;
use crate::cache::ProviderState;
use crate::error::MetricError;
use crate::models::{
    BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, GpuLiveMetrics, MemoryMetrics,
    NetworkMetrics, ProcessMetrics, SystemInfo, TemperatureMetrics,
};

pub struct LinuxProvider {
    #[allow(dead_code)]
    state: Arc<ProviderState>,
}

impl LinuxProvider {
    pub fn new(state: Arc<ProviderState>) -> Self {
        Self { state }
    }
}

impl SystemProvider for LinuxProvider {
    fn cpu(&self) -> Result<CpuMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn memory(&self) -> Result<MemoryMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn disk(&self) -> Result<DiskMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn battery(&self) -> Result<Option<BatteryMetrics>, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn energy(&self) -> Result<EnergyMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn temperature(&self) -> Result<TemperatureMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn system(&self) -> Result<SystemInfo, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn processes(&self) -> Result<ProcessMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn network(&self) -> Result<NetworkMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }

    fn gpu(&self) -> Result<GpuLiveMetrics, MetricError> {
        Err(MetricError::Unsupported(
            "LinuxProvider not implemented yet".into(),
        ))
    }
}

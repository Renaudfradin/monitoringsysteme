//! macOS [`SystemProvider`] implementation (V1).

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use sysinfo::{CpuRefreshKind, RefreshKind, System};

use super::SystemProvider;
use crate::cache::ProviderState;
use crate::error::MetricError;
use crate::models::{
    BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, MemoryMetrics, SystemInfo,
    TemperatureMetrics,
};
use crate::system::{battery, cpu, disk, energy, info, memory, temperature};

pub struct MacOSProvider {
    state: Arc<ProviderState>,
    sys: Mutex<System>,
}

impl MacOSProvider {
    pub fn new(state: Arc<ProviderState>) -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(sysinfo::MemoryRefreshKind::everything()),
        );
        Self {
            state,
            sys: Mutex::new(sys),
        }
    }
}

impl SystemProvider for MacOSProvider {
    fn cpu(&self) -> Result<CpuMetrics, MetricError> {
        let mut sys = self.sys.lock();
        // Two-sample refresh for meaningful usage percentages.
        sys.refresh_cpu_specifics(CpuRefreshKind::everything());
        thread::sleep(Duration::from_millis(120));
        cpu::collect(&mut sys)
    }

    fn memory(&self) -> Result<MemoryMetrics, MetricError> {
        let mut sys = self.sys.lock();
        memory::collect(&mut sys)
    }

    fn disk(&self) -> Result<DiskMetrics, MetricError> {
        let mut sys = self.sys.lock();
        disk::collect(&mut sys)
    }

    fn battery(&self) -> Result<Option<BatteryMetrics>, MetricError> {
        battery::collect()
    }

    fn energy(&self) -> Result<EnergyMetrics, MetricError> {
        // Compose from lightweight sibling metrics (reuses shared System).
        let cpu_m = self.cpu()?;
        let mem_m = self.memory()?;
        let bat = self.battery().unwrap_or(None);
        let model = {
            let mut sys = self.sys.lock();
            info::collect(&mut sys)
                .map(|i| i.model)
                .unwrap_or_else(|_| "Unknown".into())
        };

        let mut history = self.state.energy_history.lock();
        let metrics = energy::collect(
            energy::EnergyInputs {
                cpu: &cpu_m,
                memory: &mem_m,
                battery: &bat,
                model: &model,
            },
            &mut history,
        )?;
        *self.state.last_energy.lock() = Some(metrics.clone());
        Ok(metrics)
    }

    fn temperature(&self) -> Result<TemperatureMetrics, MetricError> {
        temperature::collect()
    }

    fn system(&self) -> Result<SystemInfo, MetricError> {
        let mut sys = self.sys.lock();
        info::collect(&mut sys)
    }
}

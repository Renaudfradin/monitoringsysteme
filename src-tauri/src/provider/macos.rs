//! macOS [`SystemProvider`] implementation (V1).

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use parking_lot::Mutex;
use sysinfo::{CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind, RefreshKind, System};

use super::SystemProvider;
use crate::cache::ProviderState;
use crate::error::MetricError;
use crate::models::{
    BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, GpuLiveMetrics, MemoryMetrics,
    NetworkMetrics, ProcessMetrics, SystemInfo, TemperatureMetrics,
};
use crate::system::{battery, cpu, disk, energy, gpu, info, memory, network, processes, temperature};

pub struct MacOSProvider {
    state: Arc<ProviderState>,
    sys: Mutex<System>,
}

impl MacOSProvider {
    pub fn new(state: Arc<ProviderState>) -> Self {
        let sys = System::new_with_specifics(
            RefreshKind::nothing()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything())
                .with_processes(ProcessRefreshKind::everything()),
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
        let metrics = cpu::collect(&mut sys)?;
        *self.state.last_cpu.lock() = Some(metrics.usage);
        Ok(metrics)
    }

    fn memory(&self) -> Result<MemoryMetrics, MetricError> {
        let mut sys = self.sys.lock();
        let metrics = memory::collect(&mut sys)?;
        *self.state.last_ram.lock() = Some(metrics.percent);
        Ok(metrics)
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
            self.system()
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
        let sys = &self.sys;
        self.state.system_info.get_or_collect(|| {
            let mut sys = sys.lock();
            info::collect(&mut sys)
        })
    }

    fn processes(&self) -> Result<ProcessMetrics, MetricError> {
        let mut sys = self.sys.lock();
        processes::collect(&mut sys)
    }

    fn network(&self) -> Result<NetworkMetrics, MetricError> {
        network::collect()
    }

    fn gpu(&self) -> Result<GpuLiveMetrics, MetricError> {
        let names: Vec<String> = self
            .system()
            .map(|s| s.gpu.into_iter().map(|g| g.name).collect())
            .unwrap_or_default();
        gpu::collect(&names)
    }
}

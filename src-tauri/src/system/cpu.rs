//! CPU metrics via sysinfo.

use sysinfo::{CpuRefreshKind, System};

use crate::error::MetricError;
use crate::models::CpuMetrics;

/// Refresh CPU and build metrics. Callers should reuse `System` when possible.
pub fn collect(sys: &mut System) -> Result<CpuMetrics, MetricError> {
    sys.refresh_cpu_specifics(CpuRefreshKind::everything());
    // Second refresh after a short sleep is done by the provider for accurate deltas.
    let cpus = sys.cpus();
    if cpus.is_empty() {
        return Err(MetricError::Unavailable("no CPU cores reported".into()));
    }

    let cores: Vec<f32> = cpus.iter().map(|c| c.cpu_usage()).collect();
    let usage = if cores.is_empty() {
        0.0
    } else {
        cores.iter().sum::<f32>() / cores.len() as f32
    };
    let frequency_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);
    let load = System::load_average();

    Ok(CpuMetrics {
        usage,
        cores,
        frequency_mhz,
        core_count: cpus.len(),
        load_avg: load.one,
    })
}

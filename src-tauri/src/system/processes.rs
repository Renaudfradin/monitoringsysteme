//! Top processes by CPU / memory.

use sysinfo::{ProcessesToUpdate, System};

use crate::error::MetricError;
use crate::models::{ProcessEntry, ProcessMetrics};

const TOP_N: usize = 8;

pub fn collect(sys: &mut System) -> Result<ProcessMetrics, MetricError> {
    sys.refresh_cpu_all();
    sys.refresh_memory();
    sys.refresh_processes(ProcessesToUpdate::All, true);

    let mut entries: Vec<ProcessEntry> = sys
        .processes()
        .iter()
        .map(|(pid, p)| ProcessEntry {
            pid: pid.as_u32(),
            name: p.name().to_string_lossy().into_owned(),
            cpu_percent: p.cpu_usage(),
            memory_bytes: p.memory(),
        })
        .collect();

    entries.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_cpu: Vec<_> = entries.iter().take(TOP_N).cloned().collect();

    entries.sort_by(|a, b| b.memory_bytes.cmp(&a.memory_bytes));
    let top_memory: Vec<_> = entries.into_iter().take(TOP_N).collect();

    Ok(ProcessMetrics {
        top_cpu,
        top_memory,
    })
}

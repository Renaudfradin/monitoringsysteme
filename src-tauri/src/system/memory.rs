//! Memory metrics via sysinfo.

use sysinfo::System;

use crate::error::MetricError;
use crate::models::MemoryMetrics;

pub fn collect(sys: &mut System) -> Result<MemoryMetrics, MetricError> {
    sys.refresh_memory();
    let total = sys.total_memory();
    let used = sys.used_memory();
    let free = total.saturating_sub(used);
    let percent = if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64 * 100.0) as f32
    };

    Ok(MemoryMetrics {
        total_bytes: total,
        used_bytes: used,
        free_bytes: free,
        percent,
    })
}

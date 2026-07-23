//! Disk metrics via sysinfo (root / primary volume).

use sysinfo::{Disks, System};

use crate::error::MetricError;
use crate::models::DiskMetrics;

pub fn collect(_sys: &mut System) -> Result<DiskMetrics, MetricError> {
    let disks = Disks::new_with_refreshed_list();
    let disk = disks
        .iter()
        .find(|d| {
            let mount = d.mount_point().to_string_lossy();
            mount == "/" || mount.eq_ignore_ascii_case("c:\\")
        })
        .or_else(|| disks.iter().next())
        .ok_or_else(|| MetricError::Unavailable("no disk found".into()))?;

    let total = disk.total_space();
    let free = disk.available_space();
    let used = total.saturating_sub(free);
    let percent = if total == 0 {
        0.0
    } else {
        (used as f64 / total as f64 * 100.0) as f32
    };

    Ok(DiskMetrics {
        total_bytes: total,
        used_bytes: used,
        free_bytes: free,
        percent,
        name: disk.name().to_string_lossy().into_owned(),
    })
}

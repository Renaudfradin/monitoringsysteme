//! Temperature / fan metrics (best effort via sysinfo components).

use sysinfo::Components;

use crate::error::MetricError;
use crate::models::TemperatureMetrics;

pub fn collect() -> Result<TemperatureMetrics, MetricError> {
    let components = Components::new_with_refreshed_list();
    if components.is_empty() {
        return Ok(TemperatureMetrics {
            cpu_celsius: None,
            gpu_celsius: None,
            ssd_celsius: None,
            fans_rpm: vec![],
            available: false,
        });
    }

    let mut cpu_celsius = None;
    let mut gpu_celsius = None;
    let mut ssd_celsius = None;
    let mut fans_rpm = Vec::new();

    for c in components.iter() {
        let label = c.label().to_lowercase();
        let Some(temp) = c.temperature() else {
            continue;
        };
        if !temp.is_finite() {
            continue;
        }
        if cpu_celsius.is_none()
            && (label.contains("cpu") || label.contains("package") || label.contains("core"))
        {
            cpu_celsius = Some(temp);
        } else if gpu_celsius.is_none() && (label.contains("gpu") || label.contains("graphics"))
        {
            gpu_celsius = Some(temp);
        } else if ssd_celsius.is_none()
            && (label.contains("ssd") || label.contains("nvme") || label.contains("disk"))
        {
            ssd_celsius = Some(temp);
        }

        if label.contains("fan") {
            // sysinfo exposes temperatures; fan RPM often unavailable — keep hook.
            fans_rpm.push(temp);
        }
    }

    let available =
        cpu_celsius.is_some() || gpu_celsius.is_some() || ssd_celsius.is_some() || !fans_rpm.is_empty();

    Ok(TemperatureMetrics {
        cpu_celsius,
        gpu_celsius,
        ssd_celsius,
        fans_rpm,
        available,
    })
}

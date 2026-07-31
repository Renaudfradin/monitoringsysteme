//! Live GPU / fan snapshot (best effort on macOS).

use sysinfo::Components;

use crate::error::MetricError;
use crate::models::{FanInfo, GpuLiveMetrics};

/// Collect live GPU utilisation proxies + fan RPM when available.
pub fn collect(gpu_names: &[String]) -> Result<GpuLiveMetrics, MetricError> {
    let components = Components::new_with_refreshed_list();

    let mut gpu_celsius = None;
    let mut fans = Vec::new();

    for (i, c) in components.iter().enumerate() {
        let label = c.label().to_string();
        let lower = label.to_lowercase();
        if let Some(temp) = c.temperature() {
            if temp.is_finite()
                && gpu_celsius.is_none()
                && (lower.contains("gpu") || lower.contains("graphics"))
            {
                gpu_celsius = Some(temp);
            }
        }

        // sysinfo rarely exposes RPM; keep structured fan slots when label matches.
        if lower.contains("fan") {
            let rpm = c.temperature().filter(|t| t.is_finite()).unwrap_or(0.0);
            fans.push(FanInfo {
                name: if label.is_empty() {
                    format!("Ventilateur {}", i + 1)
                } else {
                    label
                },
                rpm,
            });
        }
    }

    let name = gpu_names
        .first()
        .cloned()
        .unwrap_or_else(|| "GPU".into());

    // Utilisation requires Metal/IOKit — report None when unavailable.
    let available = gpu_celsius.is_some() || !fans.is_empty() || !gpu_names.is_empty();

    Ok(GpuLiveMetrics {
        name,
        utilization_percent: None,
        temperature_celsius: gpu_celsius,
        fans,
        available,
    })
}

//! Live GPU / fan snapshot (best effort).
//!
//! macOS: HID / Components temperatures. Windows: try `nvidia-smi`, then Components.

use sysinfo::Components;

use crate::error::MetricError;
use crate::models::{FanInfo, GpuLiveMetrics};

/// Collect live GPU utilisation proxies + fan RPM when available.
pub fn collect(gpu_names: &[String]) -> Result<GpuLiveMetrics, MetricError> {
    #[cfg(target_os = "windows")]
    {
        if let Some(metrics) = try_nvidia_smi(gpu_names) {
            return Ok(metrics);
        }
    }

    collect_from_components(gpu_names)
}

fn collect_from_components(gpu_names: &[String]) -> Result<GpuLiveMetrics, MetricError> {
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

    let available = gpu_celsius.is_some() || !fans.is_empty() || !gpu_names.is_empty();

    Ok(GpuLiveMetrics {
        name,
        utilization_percent: None,
        temperature_celsius: gpu_celsius,
        fans,
        available,
    })
}

#[cfg(target_os = "windows")]
fn try_nvidia_smi(gpu_names: &[String]) -> Option<GpuLiveMetrics> {
    use std::process::Command;

    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let line = text.lines().next()?.trim();
    if line.is_empty() {
        return None;
    }

    let parts: Vec<&str> = line.split(',').map(str::trim).collect();
    if parts.len() < 3 {
        return None;
    }

    let name = if parts[0].is_empty() {
        gpu_names
            .first()
            .cloned()
            .unwrap_or_else(|| "NVIDIA GPU".into())
    } else {
        parts[0].to_string()
    };
    let utilization_percent = parts[1].parse::<f32>().ok();
    let temperature_celsius = parts[2].parse::<f32>().ok();

    Some(GpuLiveMetrics {
        name,
        utilization_percent,
        temperature_celsius,
        fans: Vec::new(),
        available: true,
    })
}

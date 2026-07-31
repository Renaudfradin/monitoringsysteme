//! Temperature / fan metrics (best effort via sysinfo components).
//!
//! On Apple Silicon, HID sensors are labelled like `PMU tdie*`, `PMU2 tdie*`,
//! `NAND CH0 temp`, `gas gauge battery` — not classic `CPU` / `GPU` strings.

use sysinfo::Components;

use crate::error::MetricError;
use crate::models::{TempSensor, TemperatureMetrics};

/// Plausible laptop/desktop thermal range (°C). Filters broken HID values (e.g. -1.5).
fn is_plausible(temp: f32) -> bool {
    temp.is_finite() && (1.0..=125.0).contains(&temp)
}

fn max_of(values: &[f32]) -> Option<f32> {
    values
        .iter()
        .copied()
        .reduce(f32::max)
        .filter(|t| is_plausible(*t))
}

pub fn collect() -> Result<TemperatureMetrics, MetricError> {
    let components = Components::new_with_refreshed_list();
    if components.is_empty() {
        return Ok(TemperatureMetrics {
            cpu_celsius: None,
            gpu_celsius: None,
            ssd_celsius: None,
            battery_celsius: None,
            max_celsius: None,
            fans_rpm: vec![],
            sensors: vec![],
            available: false,
        });
    }

    let mut cpu_temps = Vec::new();
    let mut gpu_temps = Vec::new();
    let mut ssd_temps = Vec::new();
    let mut battery_temps = Vec::new();
    let mut fans_rpm = Vec::new();
    let mut sensors = Vec::new();

    for c in components.iter() {
        let label = c.label().to_string();
        let lower = label.to_lowercase();

        // Real fan RPM is rarely exposed via HID temperature services.
        if lower.contains("fan") && lower.contains("rpm") {
            if let Some(rpm) = c.temperature().filter(|t| t.is_finite() && *t > 0.0) {
                fans_rpm.push(rpm);
            }
            continue;
        }

        let Some(temp) = c.temperature().filter(|t| is_plausible(*t)) else {
            continue;
        };

        sensors.push(TempSensor {
            label: label.clone(),
            celsius: temp,
        });

        let is_pmu2 = lower.contains("pmu2");
        let is_pmu = lower.contains("pmu") && !is_pmu2;
        let is_tdie = lower.contains("tdie") || lower.contains("die");
        let is_classic_cpu =
            lower.contains("cpu") || lower.contains("package") || lower.contains("soc");
        let is_classic_gpu = lower.contains("gpu") || lower.contains("graphics");
        let is_ssd = lower.contains("nand")
            || lower.contains("ssd")
            || lower.contains("nvme")
            || lower.contains("disk");
        let is_battery = lower.contains("battery") || lower.contains("gas gauge");

        if is_ssd {
            ssd_temps.push(temp);
        } else if is_battery {
            battery_temps.push(temp);
        } else if is_classic_gpu || (is_pmu2 && is_tdie) {
            gpu_temps.push(temp);
        } else if is_classic_cpu || (is_pmu && is_tdie) {
            cpu_temps.push(temp);
        } else if is_pmu2 && !lower.contains("tdev") && !lower.contains("tcal") {
            // Secondary PMU non-ambient readings → GPU cluster proxy.
            gpu_temps.push(temp);
        }
    }

    sensors.sort_by(|a, b| {
        b.celsius
            .partial_cmp(&a.celsius)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let gpu_celsius = max_of(&gpu_temps);
    let ssd_celsius = max_of(&ssd_temps);
    let battery_celsius = max_of(&battery_temps);
    let max_celsius = sensors.first().map(|s| s.celsius);

    // Prefer die sensors; fall back to hottest reading if labels are unknown.
    let cpu_celsius = max_of(&cpu_temps).or_else(|| {
        sensors
            .iter()
            .find(|s| {
                let l = s.label.to_lowercase();
                l.contains("tdie") || l.contains("die")
            })
            .map(|s| s.celsius)
            .or(max_celsius)
    });

    let available = cpu_celsius.is_some()
        || gpu_celsius.is_some()
        || ssd_celsius.is_some()
        || battery_celsius.is_some()
        || !fans_rpm.is_empty()
        || !sensors.is_empty();

    Ok(TemperatureMetrics {
        cpu_celsius,
        gpu_celsius,
        ssd_celsius,
        battery_celsius,
        max_celsius,
        fans_rpm,
        sensors,
        available,
    })
}

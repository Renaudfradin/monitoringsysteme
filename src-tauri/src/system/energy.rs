//! Energy / power estimation.
//!
//! # Strategy (macOS V1)
//! Exact system-wide wattage typically requires elevated `powermetrics` or private SMC keys.
//! This module estimates instantaneous power from:
//! - idle baseline
//! - CPU usage × frequency ratio
//! - memory pressure proxy
//! - disk activity proxy
//! - battery charge overhead when charging
//!
//! Extension points for IOKit brightness / SMC / powermetrics are marked below.
//! `percent` is relative to an estimated TDP for the machine class.

use crate::cache::EnergyHistory;
use crate::error::MetricError;
use crate::models::{BatteryMetrics, CpuMetrics, EnergyMetrics, MemoryMetrics};

/// Default estimated thermal design power (W) when model is unknown.
const DEFAULT_TDP_W: f64 = 45.0;
const IDLE_W: f64 = 4.5;

pub struct EnergyInputs<'a> {
    pub cpu: &'a CpuMetrics,
    pub memory: &'a MemoryMetrics,
    pub battery: &'a Option<BatteryMetrics>,
    pub model: &'a str,
}

/// Estimate TDP from Mac model family (rough heuristic for % display).
pub fn estimate_tdp(model: &str) -> f64 {
    let m = model.to_lowercase();
    if m.contains("macbook air") {
        30.0
    } else if m.contains("macbook pro") && (m.contains("16") || m.contains("m3 max") || m.contains("m2 max"))
    {
        70.0
    } else if m.contains("macbook pro") {
        55.0
    } else if m.contains("mac mini") {
        40.0
    } else if m.contains("mac studio") || m.contains("mac pro") {
        120.0
    } else if m.contains("imac") {
        65.0
    } else {
        DEFAULT_TDP_W
    }
}

/// Build energy metrics and append to circular history.
pub fn collect(inputs: EnergyInputs<'_>, history: &mut EnergyHistory) -> Result<EnergyMetrics, MetricError> {
    let tdp = estimate_tdp(inputs.model);
    let cpu_ratio = (inputs.cpu.usage as f64 / 100.0).clamp(0.0, 1.0);
    // Frequency ratio: treat reported MHz against a soft max (boost-ish).
    let freq = inputs.cpu.frequency_mhz as f64;
    let freq_ratio = if freq > 0.0 {
        (freq / 3500.0).clamp(0.4, 1.2)
    } else {
        1.0
    };

    let cpu_w = cpu_ratio * freq_ratio * (tdp * 0.55);
    let mem_w = (inputs.memory.percent as f64 / 100.0) * 2.5;
    // TODO(iokit): add display brightness contribution via IOKit / CoreDisplay.
    let brightness_w = 2.0;
    // TODO(disk): refine with IOKit / IOReport disk power when available.
    let disk_w = 0.8;
    // TODO(gpu): use Metal / IOKit GPU busy when available.
    let gpu_w = cpu_ratio * 0.15 * tdp;

    let mut charge_w = 0.0;
    if let Some(bat) = inputs.battery {
        if bat.charging {
            charge_w = 8.0;
        }
    }

    let watts = (IDLE_W + cpu_w + mem_w + brightness_w + disk_w + gpu_w + charge_w).max(0.5);
    let percent = ((watts / tdp) * 100.0).clamp(0.0, 100.0) as f32;

    history.push(watts);
    let samples = history.snapshot();

    Ok(EnergyMetrics {
        watts: (watts * 10.0).round() / 10.0,
        percent,
        history: samples,
        estimated: true,
    })
}

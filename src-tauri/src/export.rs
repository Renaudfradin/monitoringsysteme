//! Export current metrics + energy history as JSON or CSV.

use crate::error::MetricError;
use crate::models::{
    BatteryMetrics, CpuMetrics, DiskMetrics, EnergyMetrics, EnergySample, GpuLiveMetrics,
    MemoryMetrics, NetworkMetrics, ProcessMetrics, SystemInfo, TemperatureMetrics,
};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportSnapshot {
    pub exported_at: u64,
    pub cpu: Option<CpuMetrics>,
    pub memory: Option<MemoryMetrics>,
    pub disk: Option<DiskMetrics>,
    pub energy: Option<EnergyMetrics>,
    pub battery: Option<BatteryMetrics>,
    pub temperature: Option<TemperatureMetrics>,
    pub processes: Option<ProcessMetrics>,
    pub network: Option<NetworkMetrics>,
    pub gpu: Option<GpuLiveMetrics>,
    pub system: Option<SystemInfo>,
}

pub fn to_json(snapshot: &ExportSnapshot) -> Result<String, MetricError> {
    serde_json::to_string_pretty(snapshot).map_err(|e| MetricError::Internal(e.to_string()))
}

pub fn to_csv(snapshot: &ExportSnapshot) -> Result<String, MetricError> {
    let mut out = String::from("section,key,value\n");
    let ts = snapshot.exported_at;
    push_row(&mut out, "meta", "exportedAt", &ts.to_string());

    if let Some(cpu) = &snapshot.cpu {
        push_row(&mut out, "cpu", "usage", &format!("{:.2}", cpu.usage));
        push_row(&mut out, "cpu", "coreCount", &cpu.core_count.to_string());
        push_row(
            &mut out,
            "cpu",
            "frequencyMhz",
            &cpu.frequency_mhz.to_string(),
        );
        push_row(&mut out, "cpu", "loadAvg", &format!("{:.3}", cpu.load_avg));
    }
    if let Some(mem) = &snapshot.memory {
        push_row(&mut out, "memory", "percent", &format!("{:.2}", mem.percent));
        push_row(&mut out, "memory", "usedBytes", &mem.used_bytes.to_string());
        push_row(&mut out, "memory", "totalBytes", &mem.total_bytes.to_string());
    }
    if let Some(disk) = &snapshot.disk {
        push_row(&mut out, "disk", "percent", &format!("{:.2}", disk.percent));
        push_row(&mut out, "disk", "usedBytes", &disk.used_bytes.to_string());
        push_row(&mut out, "disk", "totalBytes", &disk.total_bytes.to_string());
        push_row(&mut out, "disk", "name", &csv_escape(&disk.name));
    }
    if let Some(energy) = &snapshot.energy {
        push_row(&mut out, "energy", "watts", &format!("{:.2}", energy.watts));
        push_row(
            &mut out,
            "energy",
            "percent",
            &format!("{:.2}", energy.percent),
        );
        append_energy_csv(&mut out, "energy5m", &energy.history);
        append_energy_csv(&mut out, "energy24h", &energy.history_24h);
    }
    if let Some(net) = &snapshot.network {
        push_row(
            &mut out,
            "network",
            "rxBytesPerSec",
            &format!("{:.1}", net.total_rx_bytes_per_sec),
        );
        push_row(
            &mut out,
            "network",
            "txBytesPerSec",
            &format!("{:.1}", net.total_tx_bytes_per_sec),
        );
    }
    Ok(out)
}

fn append_energy_csv(out: &mut String, section: &str, samples: &[EnergySample]) {
    for s in samples {
        push_row(
            out,
            section,
            &format!("ts{}", s.ts),
            &format!("{:.2}", s.watts),
        );
    }
}

fn push_row(out: &mut String, section: &str, key: &str, value: &str) {
    out.push_str(section);
    out.push(',');
    out.push_str(key);
    out.push(',');
    out.push_str(value);
    out.push('\n');
}

fn csv_escape(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

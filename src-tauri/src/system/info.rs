//! System identity / info metrics.

use sysinfo::System;

use crate::error::MetricError;
use crate::models::SystemInfo;

pub fn collect(sys: &mut System) -> Result<SystemInfo, MetricError> {
    let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
    let os_name = System::name().unwrap_or_else(|| std::env::consts::OS.into());
    let os_version = System::os_version().unwrap_or_else(|| "unknown".into());
    let arch = std::env::consts::ARCH.to_string();
    let uptime_secs = System::uptime();
    // Prefer long OS version / kernel as a stand-in for hardware model when
    // IOKit model lookup is not wired yet.
    let model = read_model().unwrap_or_else(|| {
        System::long_os_version().unwrap_or_else(|| format!("{os_name} {os_version}"))
    });

    // Touch sys so callers can keep a shared handle warm.
    let _ = sys.cpus().len();

    Ok(SystemInfo {
        hostname,
        os_name,
        os_version,
        arch,
        uptime_secs,
        model,
    })
}

#[cfg(target_os = "macos")]
fn read_model() -> Option<String> {
    // Prefer sysctl hardware model without requiring sudo.
    use std::process::Command;
    let output = Command::new("sysctl")
        .args(["-n", "hw.model"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

#[cfg(not(target_os = "macos"))]
fn read_model() -> Option<String> {
    None
}

//! System identity, OS details, and hardware inventory.

use std::time::{Duration, Instant};

#[cfg(any(target_os = "macos", target_os = "windows"))]
use std::process::Command;

#[cfg(target_os = "macos")]
use std::collections::HashSet;

use parking_lot::Mutex;
#[cfg(any(target_os = "macos", target_os = "windows"))]
use serde_json::Value;
use sysinfo::System;

use crate::error::MetricError;
use crate::models::{CpuInfo, MemoryInfo, SystemInfo};
#[cfg(target_os = "macos")]
use crate::models::{DisplayInfo, GpuInfo, MemoryModule, StorageInfo};
#[cfg(target_os = "windows")]
use crate::models::{GpuInfo, MemoryModule, StorageInfo};

/// How long to reuse a hardware inventory snapshot (system_profiler is slow).
const INVENTORY_TTL: Duration = Duration::from_secs(120);

/// Cached static inventory; callers refresh `uptime_secs` / hostname cheaply.
pub struct SystemInfoCache {
    inner: Mutex<Option<(Instant, SystemInfo)>>,
}

impl SystemInfoCache {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(None),
        }
    }

    pub fn get_or_collect<F>(&self, collect: F) -> Result<SystemInfo, MetricError>
    where
        F: FnOnce() -> Result<SystemInfo, MetricError>,
    {
        {
            let guard = self.inner.lock();
            if let Some((at, ref cached)) = *guard {
                if at.elapsed() < INVENTORY_TTL {
                    let mut info = cached.clone();
                    info.uptime_secs = System::uptime();
                    info.hostname = System::host_name().unwrap_or_else(|| info.hostname.clone());
                    return Ok(info);
                }
            }
        }

        let info = collect()?;
        *self.inner.lock() = Some((Instant::now(), info.clone()));
        Ok(info)
    }
}

impl Default for SystemInfoCache {
    fn default() -> Self {
        Self::new()
    }
}

pub fn collect(sys: &mut System) -> Result<SystemInfo, MetricError> {
    let _ = sys.cpus().len();

    #[cfg(target_os = "macos")]
    {
        collect_macos(sys)
    }

    #[cfg(target_os = "windows")]
    {
        collect_windows(sys)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        collect_fallback(sys)
    }
}

fn collect_fallback(sys: &mut System) -> Result<SystemInfo, MetricError> {
    let hostname = System::host_name().unwrap_or_else(|| "unknown".into());
    let os_name = System::name().unwrap_or_else(|| std::env::consts::OS.into());
    let os_version = System::os_version().unwrap_or_else(|| "unknown".into());
    let os_long_name = System::long_os_version();
    let kernel_version = System::kernel_version();
    let arch = std::env::consts::ARCH.to_string();
    let uptime_secs = System::uptime();

    let brand = sys
        .cpus()
        .first()
        .map(|c| c.brand().trim().to_string())
        .filter(|s| !s.is_empty());
    let frequency_mhz = sys.cpus().first().map(|c| c.frequency()).filter(|&f| f > 0);
    let core_count = sys.cpus().len() as u32;

    let cpu = brand.map(|brand| CpuInfo {
        brand,
        vendor: None,
        cores: Some(core_count).filter(|&n| n > 0),
        performance_cores: None,
        efficiency_cores: None,
        frequency_mhz,
    });

    let model = read_hw_model().unwrap_or_else(|| {
        os_long_name
            .clone()
            .unwrap_or_else(|| format!("{os_name} {os_version}"))
    });

    Ok(SystemInfo {
        hostname,
        model,
        model_name: None,
        model_number: None,
        serial_number: None,
        hardware_uuid: None,
        firmware_version: None,
        os_name,
        os_version,
        os_build: None,
        os_long_name,
        kernel_version,
        arch,
        uptime_secs,
        cpu,
        gpu: Vec::new(),
        memory: Some(MemoryInfo {
            total_bytes: Some(sys.total_memory()).filter(|&b| b > 0),
            type_name: None,
            manufacturer: None,
            modules: Vec::new(),
        }),
        storage: Vec::new(),
        displays: Vec::new(),
    })
}

#[cfg(target_os = "macos")]
fn collect_macos(sys: &mut System) -> Result<SystemInfo, MetricError> {
    let mut info = collect_fallback(sys)?;

    if let Some(sw) = read_sw_vers() {
        if let Some(v) = sw.get("ProductName") {
            info.os_name = v.clone();
        }
        if let Some(v) = sw.get("ProductVersion") {
            info.os_version = v.clone();
        }
        if let Some(v) = sw.get("BuildVersion") {
            info.os_build = Some(v.clone());
        }
        let extra = sw.get("ProductVersionExtra").cloned();
        info.os_long_name = Some(match (info.os_build.as_ref(), extra.as_ref()) {
            (Some(build), Some(ex)) => {
                format!("{} {} {} ({})", info.os_name, info.os_version, ex, build)
            }
            (Some(build), None) => format!("{} {} ({})", info.os_name, info.os_version, build),
            (None, Some(ex)) => format!("{} {} {}", info.os_name, info.os_version, ex),
            (None, None) => format!("{} {}", info.os_name, info.os_version),
        });
    }

    if let Some(kernel) = run_capture("uname", &["-r"]) {
        info.kernel_version = Some(format!("Darwin {kernel}"));
    }

    if let Some(profiler) = read_system_profiler(&[
        "SPHardwareDataType",
        "SPDisplaysDataType",
        "SPMemoryDataType",
        "SPStorageDataType",
    ]) {
        apply_hardware_json(&mut info, &profiler);
    }

    // Prefer precise model identifier from sysctl when profiler is incomplete.
    if let Some(hw) = read_hw_model() {
        info.model = hw;
    }

    Ok(info)
}

#[cfg(target_os = "macos")]
fn apply_hardware_json(info: &mut SystemInfo, root: &Value) {
    if let Some(arr) = root.get("SPHardwareDataType").and_then(|v| v.as_array()) {
        if let Some(hw) = arr.first() {
            if let Some(s) = json_str(hw, "machine_model") {
                info.model = s;
            }
            info.model_name = json_str(hw, "machine_name");
            info.model_number = json_str(hw, "model_number");
            info.serial_number = json_str(hw, "serial_number");
            info.hardware_uuid = json_str(hw, "platform_UUID");
            info.firmware_version = json_str(hw, "boot_rom_version");

            let chip = json_str(hw, "chip_type").or_else(|| json_str(hw, "cpu_type"));
            let (cores, perf, eff) = parse_proc_topology(json_str(hw, "number_processors").as_deref());
            let mem_bytes = parse_memory_string(json_str(hw, "physical_memory").as_deref());

            if chip.is_some() || cores.is_some() {
                let brand = chip
                    .clone()
                    .or_else(|| info.cpu.as_ref().map(|c| c.brand.clone()))
                    .unwrap_or_else(|| "Unknown CPU".into());
                let vendor = if brand.to_lowercase().contains("apple") {
                    Some("Apple".into())
                } else if brand.to_lowercase().contains("intel") {
                    Some("Intel".into())
                } else if brand.to_lowercase().contains("amd") {
                    Some("AMD".into())
                } else {
                    info.cpu.as_ref().and_then(|c| c.vendor.clone())
                };
                info.cpu = Some(CpuInfo {
                    brand,
                    vendor,
                    cores: cores.or_else(|| info.cpu.as_ref().and_then(|c| c.cores)),
                    performance_cores: perf,
                    efficiency_cores: eff,
                    frequency_mhz: info.cpu.as_ref().and_then(|c| c.frequency_mhz),
                });
            }

            if let Some(total) = mem_bytes {
                let memory = info.memory.get_or_insert_with(|| MemoryInfo {
                    total_bytes: None,
                    type_name: None,
                    manufacturer: None,
                    modules: Vec::new(),
                });
                memory.total_bytes = Some(total);
            }
        }
    }

    if let Some(arr) = root.get("SPMemoryDataType").and_then(|v| v.as_array()) {
        let mut modules = Vec::new();
        let mut type_name = None;
        let mut manufacturer = None;
        let mut total = info.memory.as_ref().and_then(|m| m.total_bytes);

        for entry in arr {
            // Apple Silicon unified memory entry.
            if entry.get("dimm_type").is_some() || entry.get("SPMemoryDataType").is_some() {
                type_name = json_str(entry, "dimm_type").or(type_name);
                manufacturer = json_str(entry, "dimm_manufacturer").or(manufacturer);
                if let Some(size) = parse_memory_string(json_str(entry, "SPMemoryDataType").as_deref())
                {
                    total = Some(size);
                }
                continue;
            }

            // Classic DIMM banks under _items.
            if let Some(items) = entry.get("_items").and_then(|v| v.as_array()) {
                for dimm in items {
                    let size = parse_memory_string(json_str(dimm, "dimm_size").as_deref());
                    let speed = json_str(dimm, "dimm_speed")
                        .and_then(|s| s.replace(" MHz", "").replace("MHz", "").trim().parse().ok());
                    modules.push(MemoryModule {
                        size_bytes: size,
                        type_name: json_str(dimm, "dimm_type"),
                        speed_mhz: speed,
                        manufacturer: json_str(dimm, "dimm_manufacturer"),
                        part_number: json_str(dimm, "dimm_part_number"),
                        serial: json_str(dimm, "dimm_serial_number"),
                        slot: json_str(dimm, "_name"),
                    });
                    if type_name.is_none() {
                        type_name = json_str(dimm, "dimm_type");
                    }
                    if manufacturer.is_none() {
                        manufacturer = json_str(dimm, "dimm_manufacturer");
                    }
                }
            }
        }

        info.memory = Some(MemoryInfo {
            total_bytes: total,
            type_name,
            manufacturer,
            modules,
        });
    }

    if let Some(arr) = root.get("SPDisplaysDataType").and_then(|v| v.as_array()) {
        let mut gpus = Vec::new();
        let mut displays = Vec::new();
        for gpu in arr {
            let name = json_str(gpu, "sppci_model")
                .or_else(|| json_str(gpu, "_name"))
                .unwrap_or_else(|| "GPU".into());
            let vendor = json_str(gpu, "spdisplays_vendor")
                .or_else(|| json_str(gpu, "sppci_vendor"))
                .map(clean_sp_token);
            let cores = json_str(gpu, "sppci_cores").and_then(|s| s.parse().ok());
            let metal = json_str(gpu, "spdisplays_mtlgpufamilysupport").map(clean_sp_token);
            let bus = json_str(gpu, "sppci_bus").map(clean_sp_token);
            let vram = parse_memory_string(
                json_str(gpu, "spdisplays_vram")
                    .or_else(|| json_str(gpu, "vram_shared"))
                    .or_else(|| json_str(gpu, "spdisplays_vram_shared"))
                    .as_deref(),
            );

            gpus.push(GpuInfo {
                name: name.clone(),
                chipset: Some(name),
                vendor,
                cores,
                vram_bytes: vram,
                metal_support: metal,
                bus,
            });

            if let Some(ndrvs) = gpu.get("spdisplays_ndrvs").and_then(|v| v.as_array()) {
                for d in ndrvs {
                    let main = json_str(d, "spdisplays_main")
                        .map(|s| s.contains("yes"))
                        .unwrap_or(false);
                    displays.push(DisplayInfo {
                        name: json_str(d, "_name").unwrap_or_else(|| "Display".into()),
                        resolution: json_str(d, "_spdisplays_resolution")
                            .or_else(|| json_str(d, "spdisplays_resolution")),
                        pixel_resolution: json_str(d, "spdisplays_pixelresolution")
                            .map(clean_sp_token)
                            .or_else(|| json_str(d, "_spdisplays_pixels")),
                        display_type: json_str(d, "spdisplays_display_type").map(clean_sp_token),
                        connection: json_str(d, "spdisplays_connection_type").map(clean_sp_token),
                        vendor_id: json_str(d, "_spdisplays_display-vendor-id"),
                        product_id: json_str(d, "_spdisplays_display-product-id"),
                        serial: json_str(d, "_spdisplays_display-serial-number"),
                        main,
                    });
                }
            }
        }
        info.gpu = gpus;
        info.displays = displays;
    }

    if let Some(arr) = root.get("SPStorageDataType").and_then(|v| v.as_array()) {
        let mut by_drive: Vec<(String, StorageInfo)> = Vec::new();
        let mut seen = HashSet::new();
        for vol in arr {
            let drive = vol.get("physical_drive");
            let model = drive.and_then(|d| json_str(d, "device_name"));
            let key = model
                .clone()
                .or_else(|| json_str(vol, "bsd_name"))
                .unwrap_or_else(|| json_str(vol, "_name").unwrap_or_default());
            let entry = StorageInfo {
                name: json_str(vol, "_name").unwrap_or_else(|| "Disk".into()),
                model,
                medium_type: drive
                    .and_then(|d| json_str(d, "medium_type"))
                    .map(|s| s.to_uppercase()),
                protocol: drive.and_then(|d| json_str(d, "protocol")),
                size_bytes: vol
                    .get("size_in_bytes")
                    .and_then(|v| v.as_u64())
                    .or_else(|| {
                        drive.and_then(|d| d.get("size_in_bytes").and_then(|v| v.as_u64()))
                    }),
                serial: drive.and_then(|d| json_str(d, "serial_number")),
                smart_status: drive.and_then(|d| json_str(d, "smart_status")),
                mount_point: json_str(vol, "mount_point"),
                bsd_name: json_str(vol, "bsd_name"),
            };
            if key.is_empty() {
                by_drive.push((format!("vol-{}", by_drive.len()), entry));
                continue;
            }
            if let Some(pos) = by_drive.iter().position(|(k, _)| k == &key) {
                // Prefer the boot volume when the same physical drive appears several times.
                let prefer_new = entry.mount_point.as_deref() == Some("/");
                if prefer_new {
                    by_drive[pos] = (key, entry);
                }
            } else if seen.insert(key.clone()) {
                by_drive.push((key, entry));
            }
        }
        let mut storage: Vec<_> = by_drive.into_iter().map(|(_, s)| s).collect();
        storage.sort_by_key(|s| if s.mount_point.as_deref() == Some("/") { 0 } else { 1 });
        info.storage = storage;
    }
}

/// `number_processors` looks like `proc 8:4:4` or `proc 8:4:4:0`.
#[cfg(target_os = "macos")]
fn parse_proc_topology(raw: Option<&str>) -> (Option<u32>, Option<u32>, Option<u32>) {
    let Some(raw) = raw else {
        return (None, None, None);
    };
    let digits: Vec<u32> = raw
        .split(|c: char| !c.is_ascii_digit())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse().ok())
        .collect();
    match digits.as_slice() {
        [total, perf, eff, ..] => (Some(*total), Some(*perf), Some(*eff)),
        [total] => (Some(*total), None, None),
        _ => (None, None, None),
    }
}

#[cfg(target_os = "macos")]
fn parse_memory_string(raw: Option<&str>) -> Option<u64> {
    let raw = raw?.trim();
    if raw.is_empty() || raw.eq_ignore_ascii_case("empty") {
        return None;
    }
    let mut parts = raw.split_whitespace();
    let amount: f64 = parts.next()?.replace(',', ".").parse().ok()?;
    let unit = parts.next().unwrap_or("GB").to_ascii_uppercase();
    let mult = match unit.as_str() {
        "B" => 1.0,
        "KB" | "KIB" => 1024.0,
        "MB" | "MIB" => 1024.0 * 1024.0,
        "GB" | "GIB" => 1024.0 * 1024.0 * 1024.0,
        "TB" | "TIB" => 1024.0 * 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    Some((amount * mult) as u64)
}

#[cfg(target_os = "macos")]
fn clean_sp_token(s: String) -> String {
    s.trim()
        .trim_start_matches("spdisplays_")
        .trim_start_matches("sppci_")
        .replace('_', " ")
}

#[cfg(target_os = "macos")]
fn json_str(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| match x {
        Value::String(s) if !s.is_empty() => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

#[cfg(target_os = "macos")]
fn read_system_profiler(data_types: &[&str]) -> Option<Value> {
    let mut cmd = Command::new("system_profiler");
    cmd.args(data_types)
        .arg("-json")
        .arg("-detailLevel")
        .arg("mini");
    let output = cmd.output().ok()?;
    if !output.status.success() {
        return None;
    }
    serde_json::from_slice(&output.stdout).ok()
}

#[cfg(target_os = "macos")]
fn read_sw_vers() -> Option<std::collections::HashMap<String, String>> {
    let output = Command::new("sw_vers").output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout);
    let mut map = std::collections::HashMap::new();
    for line in text.lines() {
        if let Some((k, v)) = line.split_once(':') {
            map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    if map.is_empty() {
        None
    } else {
        Some(map)
    }
}

#[cfg(target_os = "macos")]
fn run_capture(bin: &str, args: &[&str]) -> Option<String> {
    let output = Command::new(bin).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn read_hw_model() -> Option<String> {
    #[cfg(target_os = "macos")]
    {
        return run_capture("sysctl", &["-n", "hw.model"]);
    }
    #[cfg(target_os = "linux")]
    {
        // Product name from DMI when readable.
        let name = std::fs::read_to_string("/sys/class/dmi/id/product_name")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())?;
        let version = std::fs::read_to_string("/sys/class/dmi/id/product_version")
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());
        return Some(match version {
            Some(v) if v != "None" && v != "Not Specified" => format!("{name} ({v})"),
            _ => name,
        });
    }
    #[cfg(target_os = "windows")]
    {
        return read_windows_hw_model();
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

#[cfg(target_os = "windows")]
fn collect_windows(sys: &mut System) -> Result<SystemInfo, MetricError> {
    let mut info = collect_fallback(sys)?;

    if let Some(inv) = read_windows_inventory() {
        apply_windows_inventory(&mut info, &inv);
    }

    if let Some(hw) = read_hw_model() {
        info.model = hw;
    }

    Ok(info)
}

#[cfg(target_os = "windows")]
fn read_windows_hw_model() -> Option<String> {
    let script = r#"
$cs = Get-CimInstance Win32_ComputerSystem -ErrorAction SilentlyContinue
if ($null -eq $cs) { exit 1 }
$mfr = ($cs.Manufacturer -as [string]).Trim()
$model = ($cs.Model -as [string]).Trim()
if ([string]::IsNullOrWhiteSpace($model)) { exit 1 }
if (-not [string]::IsNullOrWhiteSpace($mfr) -and $model -notlike "$mfr*") {
  Write-Output "$mfr $model"
} else {
  Write-Output $model
}
"#;
    run_powershell(script)
}

#[cfg(target_os = "windows")]
fn read_windows_inventory() -> Option<Value> {
    let script = r#"
$ErrorActionPreference = 'SilentlyContinue'
$cs = Get-CimInstance Win32_ComputerSystem
$bios = Get-CimInstance Win32_BIOS
$os = Get-CimInstance Win32_OperatingSystem
$cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$csp = Get-CimInstance Win32_ComputerSystemProduct
$cv = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
$gpus = @(Get-CimInstance Win32_VideoController | ForEach-Object {
  [pscustomobject]@{
    name = $_.Name
    adapterRam = $_.AdapterRAM
    driverVersion = $_.DriverVersion
  }
})
$mem = @(Get-CimInstance Win32_PhysicalMemory | ForEach-Object {
  [pscustomobject]@{
    capacity = $_.Capacity
    speed = $_.Speed
    manufacturer = $_.Manufacturer
    partNumber = $_.PartNumber
    serial = $_.SerialNumber
    bank = $_.BankLabel
    deviceLocator = $_.DeviceLocator
    memoryType = $_.SMBIOSMemoryType
  }
})
$disks = @(Get-CimInstance Win32_DiskDrive | ForEach-Object {
  [pscustomobject]@{
    name = $_.Caption
    model = $_.Model
    size = $_.Size
    serial = $_.SerialNumber
    interfaceType = $_.InterfaceType
    mediaType = $_.MediaType
  }
})
[pscustomobject]@{
  manufacturer = $cs.Manufacturer
  model = $cs.Model
  totalPhysicalMemory = $cs.TotalPhysicalMemory
  serial = $bios.SerialNumber
  biosVersion = $bios.SMBIOSBIOSVersion
  uuid = $csp.UUID
  osCaption = $os.Caption
  osVersion = $os.Version
  osBuild = $cv.CurrentBuild
  displayVersion = $cv.DisplayVersion
  productName = $cv.ProductName
  cpuName = $cpu.Name
  cpuCores = $cpu.NumberOfCores
  cpuLogical = $cpu.NumberOfLogicalProcessors
  cpuMaxClock = $cpu.MaxClockSpeed
  cpuManufacturer = $cpu.Manufacturer
  gpus = $gpus
  memory = $mem
  disks = $disks
} | ConvertTo-Json -Depth 5 -Compress
"#;
    let raw = run_powershell(script)?;
    serde_json::from_str(&raw).ok()
}

#[cfg(target_os = "windows")]
fn apply_windows_inventory(info: &mut SystemInfo, root: &Value) {
    if let Some(s) = json_str_any(root, "osCaption").or_else(|| json_str_any(root, "productName")) {
        info.os_name = s;
    }
    if let Some(s) = json_str_any(root, "displayVersion").or_else(|| json_str_any(root, "osVersion"))
    {
        info.os_version = s;
    }
    info.os_build = json_str_any(root, "osBuild");
    info.os_long_name = match (
        info.os_name.as_str(),
        info.os_version.as_str(),
        info.os_build.as_deref(),
    ) {
        (name, ver, Some(build)) if !name.is_empty() => Some(format!("{name} {ver} (build {build})")),
        (name, ver, None) if !name.is_empty() => Some(format!("{name} {ver}")),
        _ => info.os_long_name.clone(),
    };

    let manufacturer = json_str_any(root, "manufacturer");
    let model = json_str_any(root, "model");
    info.model = match (manufacturer.as_ref(), model.as_ref()) {
        (Some(mfr), Some(m)) if !m.to_lowercase().starts_with(&mfr.to_lowercase()) => {
            format!("{mfr} {m}")
        }
        (_, Some(m)) => m.clone(),
        (Some(mfr), None) => mfr.clone(),
        _ => info.model.clone(),
    };
    info.model_name = model;
    info.serial_number = json_str_any(root, "serial").filter(|s| {
        let l = s.to_lowercase();
        !l.is_empty() && l != "to be filled by o.e.m." && l != "default string"
    });
    info.hardware_uuid = json_str_any(root, "uuid");
    info.firmware_version = json_str_any(root, "biosVersion");

    if let Some(brand) = json_str_any(root, "cpuName") {
        let vendor = json_str_any(root, "cpuManufacturer").or_else(|| {
            let b = brand.to_lowercase();
            if b.contains("intel") {
                Some("Intel".into())
            } else if b.contains("amd") {
                Some("AMD".into())
            } else {
                None
            }
        });
        let cores = root
            .get("cpuCores")
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .or_else(|| info.cpu.as_ref().and_then(|c| c.cores));
        let frequency_mhz = root
            .get("cpuMaxClock")
            .and_then(|v| v.as_u64())
            .filter(|&f| f > 0)
            .or_else(|| info.cpu.as_ref().and_then(|c| c.frequency_mhz));
        info.cpu = Some(CpuInfo {
            brand,
            vendor,
            cores,
            performance_cores: None,
            efficiency_cores: None,
            frequency_mhz,
        });
    }

    if let Some(arr) = root.get("gpus").and_then(|v| v.as_array()) {
        let mut gpus = Vec::new();
        for gpu in arr {
            let name = json_str_any(gpu, "name").unwrap_or_else(|| "GPU".into());
            if name.to_lowercase().contains("microsoft basic") {
                continue;
            }
            let vendor = {
                let l = name.to_lowercase();
                if l.contains("nvidia") {
                    Some("NVIDIA".into())
                } else if l.contains("amd") || l.contains("radeon") {
                    Some("AMD".into())
                } else if l.contains("intel") {
                    Some("Intel".into())
                } else {
                    None
                }
            };
            let vram = gpu
                .get("adapterRam")
                .and_then(|v| v.as_u64().or_else(|| v.as_i64().map(|n| n as u64)))
                .filter(|&b| b > 0 && b < u32::MAX as u64);
            gpus.push(GpuInfo {
                name: name.clone(),
                chipset: Some(name),
                vendor,
                cores: None,
                vram_bytes: vram,
                metal_support: None,
                bus: None,
            });
        }
        if !gpus.is_empty() {
            info.gpu = gpus;
        }
    }

    if let Some(arr) = root.get("memory").and_then(|v| v.as_array()) {
        let mut modules = Vec::new();
        let mut type_name = None;
        let mut manufacturer = None;
        let mut total = root
            .get("totalPhysicalMemory")
            .and_then(|v| v.as_u64())
            .or_else(|| info.memory.as_ref().and_then(|m| m.total_bytes));

        for dimm in arr {
            let size = dimm
                .get("capacity")
                .and_then(|v| v.as_u64())
                .filter(|&b| b > 0);
            let speed = dimm
                .get("speed")
                .and_then(|v| v.as_u64())
                .map(|n| n as u32)
                .filter(|&s| s > 0);
            let mfr = json_str_any(dimm, "manufacturer");
            let mem_type = dimm
                .get("memoryType")
                .and_then(|v| v.as_u64())
                .and_then(smbios_memory_type);
            if type_name.is_none() {
                type_name = mem_type.clone();
            }
            if manufacturer.is_none() {
                manufacturer = mfr.clone();
            }
            modules.push(MemoryModule {
                size_bytes: size,
                type_name: mem_type,
                speed_mhz: speed,
                manufacturer: mfr,
                part_number: json_str_any(dimm, "partNumber"),
                serial: json_str_any(dimm, "serial"),
                slot: json_str_any(dimm, "deviceLocator")
                    .or_else(|| json_str_any(dimm, "bank")),
            });
        }

        if total.is_none() {
            let sum: u64 = modules.iter().filter_map(|m| m.size_bytes).sum();
            if sum > 0 {
                total = Some(sum);
            }
        }

        info.memory = Some(MemoryInfo {
            total_bytes: total,
            type_name,
            manufacturer,
            modules,
        });
    } else if let Some(total) = root.get("totalPhysicalMemory").and_then(|v| v.as_u64()) {
        let memory = info.memory.get_or_insert_with(|| MemoryInfo {
            total_bytes: None,
            type_name: None,
            manufacturer: None,
            modules: Vec::new(),
        });
        memory.total_bytes = Some(total);
    }

    if let Some(arr) = root.get("disks").and_then(|v| v.as_array()) {
        info.storage = arr
            .iter()
            .map(|d| StorageInfo {
                name: json_str_any(d, "name")
                    .or_else(|| json_str_any(d, "model"))
                    .unwrap_or_else(|| "Disk".into()),
                model: json_str_any(d, "model"),
                medium_type: json_str_any(d, "mediaType"),
                protocol: json_str_any(d, "interfaceType"),
                size_bytes: d.get("size").and_then(|v| v.as_u64()).filter(|&b| b > 0),
                serial: json_str_any(d, "serial"),
                smart_status: None,
                mount_point: None,
                bsd_name: None,
            })
            .collect();
    }
}

#[cfg(target_os = "windows")]
fn smbios_memory_type(code: u64) -> Option<String> {
    Some(
        match code {
            20 => "DDR",
            21 => "DDR2",
            24 => "DDR3",
            26 => "DDR4",
            34 => "DDR5",
            _ => return None,
        }
        .into(),
    )
}

#[cfg(target_os = "windows")]
fn json_str_any(v: &Value, key: &str) -> Option<String> {
    v.get(key).and_then(|x| match x {
        Value::String(s) => {
            let t = s.trim();
            if t.is_empty() {
                None
            } else {
                Some(t.to_string())
            }
        }
        Value::Number(n) => Some(n.to_string()),
        _ => None,
    })
}

#[cfg(target_os = "windows")]
fn run_powershell(script: &str) -> Option<String> {
    let output = Command::new("powershell")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-Command",
            script,
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let s = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

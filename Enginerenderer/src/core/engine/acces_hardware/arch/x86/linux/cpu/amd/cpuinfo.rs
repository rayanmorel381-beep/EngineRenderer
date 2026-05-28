//! `/proc/cpuinfo` parsing and `/sys/devices/system/cpu` sysfs helpers shared
//! between Linux x86 CPU vendors.

use std::fs;

pub(super) struct ProcCpuInfo {
    pub vendor_id: Option<String>,
    pub cpu_family: Option<u32>,
    pub model: Option<u32>,
}

pub(super) fn parse_proc_cpuinfo() -> ProcCpuInfo {
    let content = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    let mut vendor_id = None;
    let mut cpu_family = None;
    let mut model = None;

    for line in content.lines() {
        if vendor_id.is_none() && line.starts_with("vendor_id") {
            vendor_id = line.split(':').nth(1).map(|v| v.trim().to_string());
        }
        if cpu_family.is_none() && line.starts_with("cpu family") {
            cpu_family = line.split(':').nth(1).and_then(|v| v.trim().parse::<u32>().ok());
        }
        if model.is_none() && line.starts_with("model") && !line.starts_with("model name") {
            model = line.split(':').nth(1).and_then(|v| v.trim().parse::<u32>().ok());
        }
        if vendor_id.is_some() && cpu_family.is_some() && model.is_some() {
            break;
        }
    }

    let mapped_vendor = match vendor_id.as_deref() {
        Some("AuthenticAMD") => Some("AMD".to_string()),
        Some("GenuineIntel") => Some("Intel".to_string()),
        Some(v) => Some(v.to_string()),
        None => None,
    };

    ProcCpuInfo {
        vendor_id: mapped_vendor,
        cpu_family,
        model,
    }
}

pub(super) fn read_sysfs_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
}

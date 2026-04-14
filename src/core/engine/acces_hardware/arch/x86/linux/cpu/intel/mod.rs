pub(super) mod backend;
pub(super) mod scheduler;

use std::fs;

struct ProcCpuInfo {
    vendor_id: Option<String>,
    cpu_family: Option<u32>,
    model: Option<u32>,
}

fn parse_proc_cpuinfo() -> ProcCpuInfo {
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

fn read_sysfs_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
}

fn read_sysfs_string(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|v| v.trim().to_string())
}

pub(crate) struct IntelCpuInfo {
    pub cpu_family: u32,
    pub model: u32,
    pub turbo_mhz: Option<u64>,
    pub epp: Option<String>,
}

pub(crate) fn detect_intel() -> Option<IntelCpuInfo> {
    let info = parse_proc_cpuinfo();
    if info.vendor_id.as_deref() != Some("Intel") {
        return None;
    }

    let turbo_mhz = read_sysfs_u64("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .map(|khz| khz / 1000);

    let epp = read_sysfs_string(
        "/sys/devices/system/cpu/cpu0/cpufreq/energy_performance_preference",
    );

    Some(IntelCpuInfo {
        cpu_family: info.cpu_family.unwrap_or(0),
        model: info.model.unwrap_or(0),
        turbo_mhz,
        epp,
    })
}

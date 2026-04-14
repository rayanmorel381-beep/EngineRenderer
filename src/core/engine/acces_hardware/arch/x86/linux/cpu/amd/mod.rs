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

pub(crate) struct AmdCpuInfo {
    pub cpu_family: u32,
    pub model: u32,
    pub boost_mhz: Option<u64>,
    pub ccx_count: Option<u8>,
}

pub(crate) fn detect_amd() -> Option<AmdCpuInfo> {
    let info = parse_proc_cpuinfo();
    if info.vendor_id.as_deref() != Some("AMD") {
        return None;
    }

    let boost_mhz = read_sysfs_u64("/sys/devices/system/cpu/cpu0/cpufreq/cpuinfo_max_freq")
        .map(|khz| khz / 1000);

    let ccx_count = detect_amd_ccx_topology();

    Some(AmdCpuInfo {
        cpu_family: info.cpu_family.unwrap_or(0),
        model: info.model.unwrap_or(0),
        boost_mhz,
        ccx_count,
    })
}

fn detect_amd_ccx_topology() -> Option<u8> {
    let mut last_physical_id: Option<u32> = None;
    let mut count = 0u8;
    let cpuinfo = std::fs::read_to_string("/proc/cpuinfo").ok()?;
    for line in cpuinfo.lines() {
        if line.starts_with("physical id")
            && let Some(val) = line.split(':').nth(1).and_then(|v| v.trim().parse::<u32>().ok())
            && last_physical_id != Some(val)
        {
            last_physical_id = Some(val);
            count = count.saturating_add(1);
        }
    }
    if count > 0 { Some(count) } else { None }
}

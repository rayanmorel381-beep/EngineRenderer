//! Intel CPU detection on Linux via `/proc/cpuinfo` + cpufreq sysfs (turbo,
//! `energy_performance_preference`).

use super::cpuinfo::{parse_proc_cpuinfo, read_sysfs_string, read_sysfs_u64};

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

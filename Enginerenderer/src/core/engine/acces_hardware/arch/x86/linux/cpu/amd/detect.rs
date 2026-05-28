//! AMD CPU detection on Linux via `/proc/cpuinfo` + cpufreq sysfs and
//! `physical id` based CCX topology inference.

use super::cpuinfo::{parse_proc_cpuinfo, read_sysfs_u64};

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

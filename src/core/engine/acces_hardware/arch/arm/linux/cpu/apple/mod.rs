pub(super) mod backend;
pub(super) mod scheduler;

use std::fs;

pub(crate) struct ArmCpuInfo {
    pub implementer: u8,
    pub part: u16,
    pub has_neon: bool,
    pub big_little: bool,
}

pub(crate) fn detect_arm() -> Option<ArmCpuInfo> {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").ok()?;
    if !cpuinfo.contains("CPU implementer") && !cpuinfo.contains("Features") {
        return None;
    }

    let has_neon = detect_neon(&cpuinfo);
    let implementer = parse_hex_field(&cpuinfo, "CPU implementer").unwrap_or(0) as u8;
    let part = parse_hex_field(&cpuinfo, "CPU part").unwrap_or(0) as u16;
    let big_little = detect_big_little();

    Some(ArmCpuInfo {
        implementer,
        part,
        has_neon,
        big_little,
    })
}

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub(crate) fn detect_neon_arm() -> bool {
    fs::read_to_string("/proc/cpuinfo")
        .ok()
        .map(|s| detect_neon(&s))
        .unwrap_or(false)
}

fn detect_neon(cpuinfo: &str) -> bool {
    cpuinfo
        .lines()
        .any(|l| l.starts_with("Features") && l.contains("neon"))
}

fn parse_hex_field(cpuinfo: &str, field: &str) -> Option<u64> {
    cpuinfo.lines().find_map(|line| {
        if line.starts_with(field) {
            line.split(':')
                .nth(1)
                .and_then(|v| {
                    let v = v.trim().trim_start_matches("0x");
                    u64::from_str_radix(v, 16).ok()
                })
        } else {
            None
        }
    })
}

fn detect_big_little() -> bool {
    let mut unique_parts = [0u16; 8];
    let mut count = 0usize;
    let Ok(cpuinfo) = fs::read_to_string("/proc/cpuinfo") else { return false };
    for line in cpuinfo.lines() {
        if line.starts_with("CPU part")
            && let Some(val) = line.split(':').nth(1).and_then(|v| {
                let v = v.trim().trim_start_matches("0x");
                u16::from_str_radix(v, 16).ok()
            })
            && !unique_parts[..count].contains(&val)
            && count < unique_parts.len()
        {
            unique_parts[count] = val;
            count += 1;
        }
    }
    count > 1
}

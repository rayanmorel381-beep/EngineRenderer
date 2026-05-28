//! AMD CPU detection on Windows via the `CentralProcessor\0` registry key
//! (`ProcessorNameString`, `~MHz`) plus SMT inference from
//! `available_parallelism`.

use super::registry::{registry_dword, registry_string};

const CPU_KEY: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";

pub(crate) struct AmdWinInfo {
    pub brand: String,
    pub base_mhz: u32,
    pub smt_enabled: bool,
}

fn physical_cores_from_api() -> u32 {
    let logical = std::thread::available_parallelism()
        .map(|v| v.get() as u32)
        .unwrap_or(1);
    (logical / 2).max(1)
}

pub(crate) fn detect_amd() -> Option<AmdWinInfo> {
    let brand = registry_string(CPU_KEY, "ProcessorNameString")?;
    if !brand.contains("AMD") {
        return None;
    }

    let base_mhz = registry_dword(CPU_KEY, "~MHz").unwrap_or(0);

    let logical = std::thread::available_parallelism()
        .map(|v| v.get())
        .unwrap_or(1);
    let physical = physical_cores_from_api() as usize;
    let smt_enabled = physical > 0 && logical > physical;

    Some(AmdWinInfo {
        brand,
        base_mhz,
        smt_enabled,
    })
}

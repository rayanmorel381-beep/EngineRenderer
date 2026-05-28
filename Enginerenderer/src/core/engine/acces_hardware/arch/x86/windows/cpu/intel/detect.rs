//! Intel CPU detection on Windows via `CentralProcessor\0` registry key
//! (`ProcessorNameString`, `~MHz`) and PowerSettings turbo attribute lookup.

use super::registry::{registry_dword, registry_string};

const CPU_KEY: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";

pub(crate) struct IntelWinInfo {
    pub brand: String,
    pub base_mhz: u32,
    pub turbo_available: bool,
}

pub(crate) fn detect_intel() -> Option<IntelWinInfo> {
    let brand = registry_string(CPU_KEY, "ProcessorNameString")?;
    if !brand.contains("Intel") {
        return None;
    }

    let base_mhz = registry_dword(CPU_KEY, "~MHz").unwrap_or(0);

    let turbo_available = registry_dword(
        r"SYSTEM\CurrentControlSet\Control\Power\PowerSettings\54533251-82be-4824-96c1-47b60b740d00\be337238-0d82-4146-a960-4f3749d470c7",
        "Attributes",
    ).is_some();

    Some(IntelWinInfo {
        brand,
        base_mhz,
        turbo_available,
    })
}

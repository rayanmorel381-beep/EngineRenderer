//! ARM64 CPU detection on Windows via `GetSystemInfo` +
//! `CentralProcessor\0\ProcessorNameString`.

use super::registry::{PROCESSOR_ARCHITECTURE_ARM64, get_system_info, registry_string};

const CPU_KEY: &str = r"HARDWARE\DESCRIPTION\System\CentralProcessor\0";

pub(crate) struct ArmWinInfo {
    pub brand: String,
    pub core_count: u32,
}

pub(crate) fn detect_arm() -> Option<ArmWinInfo> {
    let info = get_system_info();
    if info.processor_architecture != PROCESSOR_ARCHITECTURE_ARM64 {
        return None;
    }

    let brand = registry_string(CPU_KEY, "ProcessorNameString").unwrap_or_default();

    Some(ArmWinInfo {
        brand,
        core_count: info.number_of_processors,
    })
}

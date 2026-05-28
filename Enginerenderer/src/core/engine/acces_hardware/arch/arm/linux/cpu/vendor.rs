use super::{amd, apple, intel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vendor {
    Amd,
    Intel,
    Apple,
    Unknown,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CpuConfig {
    pub(crate) vendor: Vendor,
    pub(crate) worker_hint: usize,
    pub(crate) render_workers: usize,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CpuSchedule {
    pub(crate) chunks: usize,
    pub(crate) chunk_size: usize,
    pub(crate) frame_budget_us: u64,
}

fn detect_vendor() -> Vendor {
    if let Ok(content) = std::fs::read_to_string("/proc/cpuinfo") {
        let lower = content.to_lowercase();
        if lower.contains("apple") {
            return Vendor::Apple;
        }
        if lower.contains("amd") {
            return Vendor::Amd;
        }
        if lower.contains("intel") {
            return Vendor::Intel;
        }
        if lower.contains("qualcomm")
            || lower.contains("mediatek")
            || lower.contains("exynos")
            || lower.contains("kirin")
            || lower.contains("snapdragon")
            || lower.contains("cortex")
            || lower.contains("aarch64")
            || lower.contains("armv8")
        {
            return Vendor::Unknown;
        }
    }
    Vendor::Unknown
}

pub(crate) fn default_config() -> CpuConfig {
    let arm_probe = apple::detect::detect_arm();
    let has_neon_fallback = apple::detect::detect_neon_arm();
    let (implementer, part, big_little, has_neon_from_probe) = match arm_probe {
        Some(info) => (info.implementer, info.part, info.big_little, info.has_neon),
        None => (0u8, 0u16, false, false),
    };
    let has_neon = has_neon_from_probe || has_neon_fallback;
    let tuning_hint = usize::from(implementer) + usize::from(part) + usize::from(big_little);
    let vendor = detect_vendor();
    let (worker_hint, render_workers, frame_budget_us, low_power) = match vendor {
        Vendor::Amd => {
            let c = amd::backend::default_backend_config();
            (c.worker_hint, c.render_workers, c.frame_budget_us, c.low_power || !has_neon)
        }
        Vendor::Intel => {
            let c = intel::backend::default_backend_config();
            (c.worker_hint, c.render_workers, c.frame_budget_us, c.low_power || !has_neon)
        }
        Vendor::Apple => {
            let c = apple::backend::default_backend_config();
            (
                c.worker_hint.max((tuning_hint % 4) + 1),
                c.render_workers,
                c.frame_budget_us,
                c.low_power || big_little || !has_neon,
            )
        }
        Vendor::Unknown => {
            let total = std::thread::available_parallelism()
                .map(|v| v.get())
                .unwrap_or(1)
                .max(1);
            let render_workers = total.saturating_sub(1).max(1);
            (
                total.max((tuning_hint % 4) + 1),
                render_workers,
                8_333,
                big_little || !has_neon,
            )
        }
    };
    CpuConfig { vendor, worker_hint, render_workers, frame_budget_us, low_power }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    match detect_vendor() {
        Vendor::Amd   => amd::backend::clamp_workers(requested),
        Vendor::Intel => intel::backend::clamp_workers(requested),
        Vendor::Apple => apple::backend::clamp_workers(requested),
        Vendor::Unknown => requested.max(1).min(std::thread::available_parallelism().map(|v| v.get()).unwrap_or(1).max(1)),
    }
}

pub(crate) fn build_schedule(work_items: usize) -> CpuSchedule {
    let vendor = detect_vendor();
    let (chunks, chunk_size, frame_budget_us) = match vendor {
        Vendor::Amd => {
            let s = amd::scheduler::build_schedule(work_items);
            (s.chunks, s.chunk_size, s.frame_budget_us)
        }
        Vendor::Intel => {
            let s = intel::scheduler::build_schedule(work_items);
            (s.chunks, s.chunk_size, s.frame_budget_us)
        }
        Vendor::Apple => {
            let s = apple::scheduler::build_schedule(work_items);
            (s.chunks, s.chunk_size, s.frame_budget_us)
        }
        Vendor::Unknown => {
            let chunk_size = if work_items == 0 { 32 } else { work_items.div_ceil(32).max(32) };
            let chunks = if work_items == 0 { 1 } else { work_items.div_ceil(chunk_size) };
            (chunks, chunk_size, 8_333)
        }
    };
    CpuSchedule { chunks, chunk_size, frame_budget_us }
}

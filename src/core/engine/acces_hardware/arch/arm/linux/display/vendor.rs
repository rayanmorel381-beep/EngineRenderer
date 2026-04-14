use super::{amd, apple, intel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vendor {
    Amd,
    Intel,
    Apple,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DisplayConfig {
    pub(crate) vendor: Vendor,
    pub(crate) page_size: usize,
    pub(crate) target_render_fps: u32,
    pub(crate) latency_budget_us: u64,
    pub(crate) scan_out_latency_us: u64,
    pub(crate) vsync_slots: usize,
    pub(crate) double_buffered: bool,
    pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DisplaySchedule {
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
    }
    Vendor::Apple
}

pub(crate) fn default_config() -> DisplayConfig {
    let vendor = detect_vendor();
    let (page_size, target_render_fps, latency_budget_us, scan_out_latency_us, vsync_slots, double_buffered, low_power) = match vendor {
        Vendor::Amd => {
            let c = amd::backend::default_backend_config();
            (c.page_size, c.target_render_fps, c.latency_budget_us, c.scan_out_latency_us, c.vsync_slots, c.double_buffered, c.low_power)
        }
        Vendor::Intel => {
            let c = intel::backend::default_backend_config();
            (c.page_size, c.target_render_fps, c.latency_budget_us, c.scan_out_latency_us, c.vsync_slots, c.double_buffered, c.low_power)
        }
        Vendor::Apple => {
            let c = apple::backend::default_backend_config();
            (c.page_size, c.target_render_fps, c.latency_budget_us, c.scan_out_latency_us, c.vsync_slots, c.double_buffered, c.low_power)
        }
    };
    DisplayConfig { vendor, page_size, target_render_fps, latency_budget_us, scan_out_latency_us, vsync_slots, double_buffered, low_power }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    match detect_vendor() {
        Vendor::Amd   => amd::backend::clamp_workers(requested),
        Vendor::Intel => intel::backend::clamp_workers(requested),
        Vendor::Apple => apple::backend::clamp_workers(requested),
    }
}

pub(crate) fn build_schedule(work_items: usize) -> DisplaySchedule {
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
    };
    DisplaySchedule { chunks, chunk_size, frame_budget_us }
}

use super::{amd, apple, intel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vendor {
    Amd,
    Intel,
    Apple,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuConfig {
    pub(crate) vendor: Vendor,
    pub(crate) workgroup_size: usize,
    pub(crate) compute_queues: usize,
    pub(crate) render_threads: usize,
    pub(crate) double_buffered: bool,
    pub(crate) frame_budget_us: u64,
    pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuSchedule {
    pub(crate) chunks: usize,
    pub(crate) chunk_size: usize,
    pub(crate) frame_budget_us: u64,
}

fn detect_vendor() -> Vendor {
    let amd_probes: [amd::GpuProbeResult; 0] = [];
    if amd::probe(&amd_probes).is_some() {
        return Vendor::Amd;
    }
    let intel_probes: [intel::GpuProbeResult; 0] = [];
    if intel::probe(&intel_probes).is_some() {
        return Vendor::Intel;
    }
    if let Ok(id) = std::env::var("PROCESSOR_IDENTIFIER") {
        let id = id.to_lowercase();
        if id.contains("apple") {
            return Vendor::Apple;
        }
        if id.contains("amd") {
            return Vendor::Amd;
        }
        if id.contains("intel") {
            return Vendor::Intel;
        }
    }
    Vendor::Intel
}

pub(crate) fn default_config() -> GpuConfig {
    let vendor = detect_vendor();
    let (workgroup_size, compute_queues, render_threads, double_buffered, frame_budget_us, low_power) = match vendor {
        Vendor::Amd => {
            let c = amd::backend::default_backend_config();
            (c.workgroup_size, c.compute_queues, c.render_threads, c.double_buffered, c.frame_budget_us, c.low_power)
        }
        Vendor::Intel => {
            let c = intel::backend::default_backend_config();
            (c.workgroup_size, c.compute_queues, c.render_threads, c.double_buffered, c.frame_budget_us, c.low_power)
        }
        Vendor::Apple => {
            let c = apple::backend::default_backend_config();
            (c.workgroup_size, c.compute_queues, c.render_threads, c.double_buffered, c.frame_budget_us, c.low_power)
        }
    };
    GpuConfig { vendor, workgroup_size, compute_queues, render_threads, double_buffered, frame_budget_us, low_power }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    match detect_vendor() {
        Vendor::Amd   => amd::backend::clamp_workers(requested),
        Vendor::Intel => intel::backend::clamp_workers(requested),
        Vendor::Apple => apple::backend::clamp_workers(requested),
    }
}

pub(crate) fn build_schedule(work_items: usize) -> GpuSchedule {
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
    GpuSchedule { chunks, chunk_size, frame_budget_us }
}

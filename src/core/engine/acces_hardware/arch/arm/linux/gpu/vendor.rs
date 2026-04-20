use super::{amd, apple, intel};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vendor {
    Amd,
    Intel,
    Apple,
    Unknown,
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
    if let Ok(driver) = std::fs::read_to_string("/sys/class/drm/card0/device/driver/module/drivers") {
        let lower = driver.to_lowercase();
        if lower.contains("amdgpu") || lower.contains("radeon") {
            return Vendor::Amd;
        }
        if lower.contains("i915") || lower.contains("xe") {
            return Vendor::Intel;
        }
        if lower.contains("apple") || lower.contains("agx") {
            return Vendor::Apple;
        }
    }
    Vendor::Unknown
}

fn probe_gpu_runtime(vendor: Vendor) {
    match vendor {
        Vendor::Amd => {
            let (cu_a, se_a, sclk_a, temp_a) = amd::drm::probe_amdgpu_telemetry("card0");
            let (cu_r, se_r, sclk_r, temp_r) = amd::drm::probe_radeon_telemetry("card0");
            let target_size = ((cu_a.max(cu_r) as u64).max(1))
                .saturating_mul((sclk_a.max(sclk_r) as u64).max(1));
            if let Some(gem) = amd::drm::drm_amdgpu_alloc_gem(-1, target_size.max(4096)) {
                let mapped = amd::drm::drm_amdgpu_gem_mmap(gem.fd, gem.handle).unwrap_or(0);
                let effective_map = mapped.max(gem.mmap_offset);
                let wait_ok = amd::drm::drm_amdgpu_wait_cs(gem.fd, 0, 1_000_000);
                if !wait_ok {
                    crate::runtime_log!("gpu: amdgpu wait_cs failed");
                }
                if let Err(err) = amd::drm::submit_amdgpu_cs(
                    gem.fd,
                    gem.handle,
                    &[0, cu_a, se_a, sclk_a, temp_a as u32, effective_map as u32],
                ) {
                    crate::runtime_log!("gpu: amdgpu submit failed: {}", err);
                }
            }
            if let Some(gem) = amd::drm::drm_radeon_alloc_gem(-1, target_size.max(4096)) {
                let maybe_mapped = amd::drm::drm_radeon_gem_mmap(gem.fd, gem.handle, gem.size);
                if maybe_mapped.is_none() {
                    crate::runtime_log!("gpu: radeon mmap failed");
                }
                let wait_ok = amd::drm::drm_radeon_gem_wait(gem.fd, gem.handle);
                if !wait_ok {
                    crate::runtime_log!("gpu: radeon wait failed");
                }
                if let Err(err) = amd::drm::submit_radeon_cs(gem.fd, gem.handle, &[0, cu_r, se_r, sclk_r, temp_r as u32]) {
                    crate::runtime_log!("gpu: radeon submit failed: {}", err);
                }
            }
        }
        Vendor::Intel => {
            let (eu, slices, freq, temp) = intel::drm::probe_i915_telemetry("card0");
            let target_size = ((eu.max(1) as u64)
                .saturating_mul(slices.max(1) as u64)
                .saturating_mul(freq.max(1) as u64))
                .max(4096);
            if let Some(gem) = intel::drm::drm_i915_alloc_gem(-1, target_size) {
                let mapped = intel::drm::drm_i915_gem_mmap_gtt(gem.fd, gem.handle).unwrap_or(0);
                let effective_size = gem.size.max(gem.mmap_offset).max(mapped);
                let wait_ok = intel::drm::drm_i915_gem_wait(gem.fd, gem.handle, 1_000_000);
                if !wait_ok {
                    crate::runtime_log!("gpu: i915 wait failed");
                }
                if let Err(err) = intel::drm::submit_i915_execbuf(
                    gem.fd,
                    gem.handle,
                    &[0, eu, slices, freq, temp as u32, effective_size as u32],
                ) {
                    crate::runtime_log!("gpu: i915 submit failed: {}", err);
                }
            }
        }
        Vendor::Apple => {}
        Vendor::Unknown => {}
    }
}

pub(crate) fn default_config() -> GpuConfig {
    let vendor = detect_vendor();
    probe_gpu_runtime(vendor);
    let (workgroup_size, compute_queues, render_threads, double_buffered, frame_budget_us, low_power) = match vendor {
        Vendor::Amd => {
            let c = amd::backend::default_backend_config();
            (
                c.workgroup_size,
                c.compute_queues,
                c.render_threads,
                c.double_buffered,
                c.frame_budget_us,
                c.low_power,
            )
        }
        Vendor::Intel => {
            let c = intel::backend::default_backend_config();
            (
                c.workgroup_size,
                c.compute_queues,
                c.render_threads,
                c.double_buffered,
                c.frame_budget_us,
                c.low_power,
            )
        }
        Vendor::Apple => {
            let c = apple::backend::default_backend_config();
            (c.workgroup_size, c.compute_queues, c.render_threads, c.double_buffered, c.frame_budget_us, c.low_power)
        }
        Vendor::Unknown => {
            let workers = std::thread::available_parallelism()
                .map(|v| v.get())
                .unwrap_or(1)
                .max(1);
            (32, workers.min(4), workers.min(16), true, 8_333, false)
        }
    };
    GpuConfig { vendor, workgroup_size, compute_queues, render_threads, double_buffered, frame_budget_us, low_power }
}

pub(crate) fn clamp_workers(requested: usize) -> usize {
    match detect_vendor() {
        Vendor::Amd   => amd::backend::clamp_workers(requested),
        Vendor::Intel => intel::backend::clamp_workers(requested),
        Vendor::Apple => apple::backend::clamp_workers(requested),
        Vendor::Unknown => requested.clamp(1, 32),
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
        Vendor::Unknown => {
            let chunk_size = if work_items == 0 { 64 } else { work_items.div_ceil(64).max(64) };
            let chunks = if work_items == 0 { 1 } else { work_items.div_ceil(chunk_size) };
            (chunks, chunk_size, 8_333)
        }
    };
    GpuSchedule { chunks, chunk_size, frame_budget_us }
}

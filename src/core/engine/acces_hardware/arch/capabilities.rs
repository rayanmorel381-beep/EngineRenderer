use super::compute_dispatch;

const GPU_MAX_PERCENT: u64 = 80;
const CPU_MAX_PERCENT: usize = 80;
const RAM_MAX_PERCENT: u64 = 50;
const BYTES_PER_PIXEL_F64_RGB: u64 = 24;

#[derive(Debug, Clone)]
pub struct HardwareCapabilities {
    pub logical_cores: u32,
    pub vram_bytes: u64,
    total_ram_bytes: u64,
}

impl HardwareCapabilities {
    pub fn detect() -> Self {
        let config = compute_dispatch::default_config();
        let logical_cores = std::thread::available_parallelism()
            .map(|n| n.get() as u32)
            .unwrap_or(1);
        let total_ram_bytes = config.ram.total_bytes;
        let vram_bytes = detect_vram_bytes();
        Self { logical_cores, vram_bytes, total_ram_bytes }
    }

    pub fn optimal_render_threads(&self) -> usize {
        self.optimal_render_threads_for_input(usize::MAX)
    }

    pub fn optimal_render_threads_for_input(&self, input_work_items: usize) -> usize {
        let logical = (self.logical_cores as usize).max(1);
        let cpu_cap = ((logical * CPU_MAX_PERCENT) / 100).max(1);
        let os_cap = compute_dispatch::default_cpu_config().render_workers.max(1);
        let input_cap = input_work_items.max(1).div_ceil(4_000).max(1);
        cpu_cap.min(os_cap).min(input_cap).max(1)
    }

    pub fn max_framebuffer_bytes(&self) -> u64 {
        if self.total_ram_bytes == 0 {
            512 * 1024 * 1024
        } else {
            (self.total_ram_bytes.saturating_mul(RAM_MAX_PERCENT)) / 100
        }
    }

    pub fn max_framebuffer_bytes_for_input(&self, input_pixels: usize) -> u64 {
        let requested = (input_pixels as u64).saturating_mul(BYTES_PER_PIXEL_F64_RGB);
        requested.min(self.max_framebuffer_bytes()).max(1)
    }

    pub fn max_gpu_allocation_bytes(&self) -> u64 {
        if self.vram_bytes == 0 {
            0
        } else {
            (self.vram_bytes.saturating_mul(GPU_MAX_PERCENT)) / 100
        }
    }

    pub fn log_summary(&self) {
        eprintln!(
            "hardware: logical_cores={} vram={}MB ram={}MB",
            self.logical_cores,
            self.vram_bytes / (1024 * 1024),
            self.total_ram_bytes / (1024 * 1024),
        );
    }
}

fn detect_vram_bytes() -> u64 {
    #[cfg(target_os = "linux")]
    {
        for i in 0..8 {
            let path = format!("/sys/class/drm/card{}/device/mem_info_vram_total", i);
            if let Ok(s) = std::fs::read_to_string(&path)
                && let Ok(n) = s.trim().parse::<u64>()
                && n > 0
            {
                return n;
            }
        }
    }
    0
}

//! GPU render backend — DRM-based GPU buffer management and compute
//! dispatch for tile-parallel ray tracing.
//!
//! Opens the GPU via DRM, allocates GEM buffers in GTT (system memory
//! accessible to GPU), and submits instruction buffers via ioctl.

use hardware::sys;

/// GPU rendering backend using the hardware crate's DRM interface.
pub struct GpuRenderBackend {
    drm: sys::gpu::drm::DrmDevice,
    info: sys::gpu::drm::GpuInfo,
    framebuffer: Option<sys::gpu::drm::GemBuffer>,
}

impl std::fmt::Debug for GpuRenderBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GpuRenderBackend")
            .field("device_id", &self.info.device_id)
            .field("vram_bytes", &self.info.vram_bytes)
            .field("compute_units", &self.info.active_cu)
            .field("has_framebuffer", &self.framebuffer.is_some())
            .finish()
    }
}

impl GpuRenderBackend {
    /// Attempts to open the DRM device and probe GPU capabilities.
    ///
    /// Returns `None` if no GPU is available or DRM open fails.
    pub fn try_init() -> Option<Self> {
        let drm = sys::gpu::drm::open()?;
        let info = drm.query_gpu_info();
        let driver = match drm.driver {
            sys::gpu::drm::DrmDriver::Radeon => "radeon",
            sys::gpu::drm::DrmDriver::Amdgpu => "amdgpu",
            sys::gpu::drm::DrmDriver::Nouveau => "nouveau",
            sys::gpu::drm::DrmDriver::I915 => "i915",
            sys::gpu::drm::DrmDriver::Unknown => "unknown",
        };

        if info.device_id == 0 && info.vram_bytes == 0 && info.active_cu == 0 {
            eprintln!(
                "gpu: drm driver={} active, but detailed telemetry is unavailable from the hardware crate",
                driver,
            );
        } else {
            eprintln!(
                "gpu: driver={} device={:04x} vram={}MB cu={} se={} sclk={}MHz temp={}°C",
                driver,
                info.device_id,
                info.vram_bytes / (1024 * 1024),
                info.active_cu,
                info.shader_engines,
                info.gpu_sclk_mhz,
                info.gpu_temp,
            );
        }
        Some(Self {
            drm,
            info,
            framebuffer: None,
        })
    }

    /// Allocate a GPU-accessible framebuffer in GTT (GART) memory.
    ///
    /// The returned buffer is mmap'd into CPU address space so the ray
    /// tracer can write directly into GPU-visible memory with zero copies.
    pub fn alloc_framebuffer(&mut self, width: usize, height: usize) -> Option<*mut u8> {
        let pixel_bytes = (width * height * 3 * 8) as u64; // 3 channels × f64
        let mut buf = self.drm.gem_create_gtt(pixel_bytes)?;
        if !self.drm.gem_mmap(&mut buf) {
            self.drm.gem_close(&mut buf);
            return None;
        }
        let ptr = buf.mapped;
        self.framebuffer = Some(buf);
        Some(ptr)
    }

    /// Submit a raw instruction buffer to the GPU for execution.
    ///
    /// `ib_data` is a slice of PM4 packets (DWORD-aligned GPU commands).
    /// Returns the command submission ID from the kernel DRM driver.
    pub fn submit_ib(&self, ib_data: &[u32]) -> Result<i64, &'static str> {
        if ib_data.is_empty() {
            return Err("empty instruction buffer");
        }
        let cs_id = self.drm.submit_ib(
            ib_data.as_ptr(),
            ib_data.len() as u32,
        );
        if cs_id < 0 {
            return Err("DRM command submission failed");
        }
        Ok(cs_id)
    }

    /// Wait for the GPU to finish all pending work on the framebuffer.
    pub fn sync_framebuffer(&self) {
        if let Some(ref buf) = self.framebuffer {
            self.drm.gem_wait_idle(buf);
        }
    }

    /// Whether a GEM framebuffer is allocated and mapped.
    pub fn has_active_framebuffer(&self) -> bool {
        self.framebuffer.is_some()
    }

    /// GPU info — VRAM, compute units, shader engines, clocks.
    pub fn info(&self) -> &sys::gpu::drm::GpuInfo {
        &self.info
    }

    pub fn driver_name(&self) -> &'static str {
        match self.drm.driver {
            sys::gpu::drm::DrmDriver::Radeon => "radeon",
            sys::gpu::drm::DrmDriver::Amdgpu => "amdgpu",
            sys::gpu::drm::DrmDriver::Nouveau => "nouveau",
            sys::gpu::drm::DrmDriver::I915 => "i915",
            sys::gpu::drm::DrmDriver::Unknown => "unknown",
        }
    }

    pub fn has_valid_metrics(&self) -> bool {
        self.info.device_id != 0 || self.info.vram_bytes != 0 || self.info.active_cu != 0
    }

    /// VRAM in bytes.
    pub fn vram_bytes(&self) -> u64 {
        self.info.vram_bytes
    }

    /// Number of active compute units.
    pub fn compute_units(&self) -> u32 {
        self.info.active_cu
    }
}

impl Drop for GpuRenderBackend {
    fn drop(&mut self) {
        if let Some(ref mut buf) = self.framebuffer {
            self.drm.gem_close(buf);
        }
    }
}

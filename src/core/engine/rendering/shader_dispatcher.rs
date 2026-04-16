//! Compute device abstraction and adaptive dispatch utilities.

use crate::core::engine::acces_hardware::{
    ComputeCapabilities, ComputeDeviceKind, ComputeJobBatch, ComputeQueue, GpuRenderBackend, KernelConfig,
    NativeComputeBackend, NativeHardwareBackend,
};
use crate::core::engine::acces_hardware::arch::native_calls;

/// Trait implemented by all compute backends used by the renderer.
pub trait ComputeDevice: Send + Sync {
    /// Returns static capabilities for this compute device.
    fn capabilities(&self) -> ComputeCapabilities;

    /// Compiles a kernel source for the concrete backend.
    fn compile_kernel(
        &self,
        name: &str,
        kernel_source: &[u8],
    ) -> Result<Vec<u8>, String>;

    /// Submits a prepared job batch to the backend queue.
    fn submit_batch(&self, batch: &ComputeJobBatch) -> Result<u64, String>;

    /// Waits until the backend queue becomes idle.
    fn wait_idle(&self);

    /// Returns a human-readable backend name.
    fn device_name(&self) -> &str;
}

/// Generic compute device implementation for CPU and GPU paths.
pub struct GenericComputeDevice {
    capabilities: ComputeCapabilities,
    backend: NativeComputeBackend,
}

impl GenericComputeDevice {
    pub fn from_native_backend(native: &NativeHardwareBackend, kind: ComputeDeviceKind) -> Self {
        Self {
            capabilities: native.capabilities_for(kind),
            backend: native.create_compute_backend(kind),
        }
    }

    /// Creates a GPU device with an initialized native submission path.
    pub fn new_gpu_with_fd(
        backend: &GpuRenderBackend,
        max_workgroups: u32,
        max_workgroup_size: u32,
        parallel_lanes: u32,
        shared_memory_bytes: u32,
    ) -> Self {
        let device_name = format!(
            "GPU-{}-{}cu",
            backend.driver().name(),
            backend.compute_units(),
        );

        Self {
            capabilities: ComputeCapabilities::gpu(
                max_workgroups,
                max_workgroup_size,
                parallel_lanes,
                shared_memory_bytes,
            ),
            backend: NativeComputeBackend::new(
                device_name,
                ComputeDeviceKind::Gpu,
                parallel_lanes,
                backend.drm_fd(),
                Some(crate::core::engine::acces_hardware::GpuSubmitter::new(
                    backend.drm_fd(),
                    backend.driver(),
                    backend.gem_handle(),
                )),
            ),
        }
    }

    /// Creates a SIMD-optimized CPU device.
    pub fn new_cpu_simd() -> Self {
        Self {
            capabilities: ComputeCapabilities::cpu_simd(),
            backend: NativeComputeBackend::new(
                "CPU-SIMD".to_string(),
                ComputeDeviceKind::CpuSimd,
                ComputeCapabilities::cpu_simd().parallel_lanes,
                -1,
                None,
            ),
        }
    }

    /// Creates a scalar CPU fallback device.
    pub fn new_cpu_scalar() -> Self {
        Self {
            capabilities: ComputeCapabilities::cpu_scalar(),
            backend: NativeComputeBackend::new(
                "CPU-Scalar".to_string(),
                ComputeDeviceKind::CpuScalar,
                ComputeCapabilities::cpu_scalar().parallel_lanes,
                -1,
                None,
            ),
        }
    }
}

impl ComputeDevice for GenericComputeDevice {
    fn capabilities(&self) -> ComputeCapabilities {
        self.capabilities
    }

    fn compile_kernel(
        &self,
        name: &str,
        kernel_source: &[u8],
    ) -> Result<Vec<u8>, String> {
        self.backend.compile_kernel(name, kernel_source)
    }

    fn submit_batch(&self, batch: &ComputeJobBatch) -> Result<u64, String> {
        self.backend.submit_batch(batch)
    }

    fn wait_idle(&self) {
        self.backend.wait_idle();
    }

    fn device_name(&self) -> &str {
        self.backend.device_name()
    }
}

/// Adaptive dispatcher that selects and drives compute devices.
pub struct AdaptiveComputeDispatcher {
    devices: Vec<Box<dyn ComputeDevice>>,
    active_device_idx: usize,
}

pub struct TileComputeDescriptor<'a> {
    pub image_width: usize,
    pub image_height: usize,
    pub tile_size: usize,
    pub config: KernelConfig,
    pub kernel_name: &'a str,
    pub kernel_source: &'a [u8],
    pub scene_signature: u64,
    pub object_count: usize,
    pub triangle_count: usize,
}

impl AdaptiveComputeDispatcher {
    /// Creates an empty dispatcher.
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            active_device_idx: 0,
        }
    }

    /// Registers a compute device into the dispatcher.
    pub fn register_device(&mut self, device: Box<dyn ComputeDevice>) {
        eprintln!("compute: registered device '{}' — {}", device.device_name(), self.device_count() + 1);
        self.devices.push(device);
    }

    /// Creates a dispatcher and auto-registers available devices.
    pub fn with_auto_detection(gpu_backend: Option<&GpuRenderBackend>) -> Self {
        let mut dispatcher = Self::new();

        if let Some(backend) = gpu_backend {
            let gpu_device = Box::new(GenericComputeDevice::new_gpu_with_fd(
                backend,
                backend.info().active_cu * 4,
                256,
                128,
                65536,
            ));
            dispatcher.register_device(gpu_device);
            dispatcher.active_device_idx = 0;
        }

        let has_simd = native_calls::host_has_simd();

        let cpu_device = if has_simd {
            Box::new(GenericComputeDevice::new_cpu_simd()) as Box<dyn ComputeDevice>
        } else {
            Box::new(GenericComputeDevice::new_cpu_scalar()) as Box<dyn ComputeDevice>
        };
        dispatcher.register_device(cpu_device);

        dispatcher
    }

    pub fn with_native_backend(native: &NativeHardwareBackend) -> Self {
        let mut dispatcher = Self::new();

        if native.gpu_backend().is_some() {
            dispatcher.register_device(Box::new(GenericComputeDevice::from_native_backend(
                native,
                ComputeDeviceKind::Gpu,
            )));
            dispatcher.active_device_idx = 0;
        }

        let has_simd = native_calls::host_has_simd();
        let cpu_kind = if has_simd {
            ComputeDeviceKind::CpuSimd
        } else {
            ComputeDeviceKind::CpuScalar
        };
        dispatcher.register_device(Box::new(GenericComputeDevice::from_native_backend(
            native,
            cpu_kind,
        )));

        dispatcher
    }

    /// Selects the active device by index.
    pub fn set_active_device(&mut self, device_idx: usize) -> Result<(), String> {
        if device_idx >= self.devices.len() {
            return Err(format!("device index {} out of range", device_idx));
        }
        self.active_device_idx = device_idx;
        eprintln!(
            "compute: switched to device '{}'",
            self.devices[device_idx].device_name()
        );
        Ok(())
    }

    /// Returns an immutable reference to the active compute device.
    pub fn active_device(&self) -> &dyn ComputeDevice {
        &*self.devices[self.active_device_idx]
    }

    /// Returns a mutable reference to the active compute device.
    pub fn active_device_mut(&mut self) -> &mut dyn ComputeDevice {
        &mut *self.devices[self.active_device_idx]
    }

    /// Returns the number of registered devices.
    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

    /// Lists registered devices and their capability summaries.
    pub fn list_devices(&self) -> Vec<String> {
        self.devices
            .iter()
            .enumerate()
            .map(|(i, d)| {
                let caps = d.capabilities();
                format!(
                    "{}. {} — {} lanes, {} workgroups max",
                    i,
                    d.device_name(),
                    caps.parallel_lanes,
                    caps.max_workgroups
                )
            })
            .collect()
    }

    /// Builds and dispatches tile jobs for the current active device.
    pub fn dispatch_tile_compute(
        &mut self,
        image_width: usize,
        image_height: usize,
        tile_size: usize,
        config: KernelConfig,
    ) -> Result<u64, String> {
        self.dispatch_tile_compute_with_kernel(TileComputeDescriptor {
            image_width,
            image_height,
            tile_size,
            config,
            kernel_name: "generic-tile",
            kernel_source: b"kernel generic_tile(u32 tile_id) { return; }",
            scene_signature: 0,
            object_count: 0,
            triangle_count: 0,
        })
    }

    pub fn dispatch_tile_compute_with_kernel(
        &mut self,
        descriptor: TileComputeDescriptor<'_>,
    ) -> Result<u64, String> {
        let device = self.active_device();
        let mut batch = ComputeJobBatch::new(device.capabilities().max_workgroups as usize);
        let binary = device.compile_kernel(descriptor.kernel_name, descriptor.kernel_source)?;
        batch.set_metadata(
            hash32(&binary),
            binary.len() as u32,
            descriptor.scene_signature,
            descriptor.object_count.min(u32::MAX as usize) as u32,
            descriptor.triangle_count.min(u32::MAX as usize) as u32,
        );

        let tiles_across = descriptor.image_width.div_ceil(descriptor.tile_size);
        let tiles_down = descriptor.image_height.div_ceil(descriptor.tile_size);

        for tile_y in 0..tiles_down {
            for tile_x in 0..tiles_across {
                let tile_id = (tile_y * tiles_across + tile_x) as u32;
                let x_min = tile_x * descriptor.tile_size;
                let y_min = tile_y * descriptor.tile_size;
                let x_max = (x_min + descriptor.tile_size).min(descriptor.image_width);
                let y_max = (y_min + descriptor.tile_size).min(descriptor.image_height);

                let pixel_count = (x_max - x_min) * (y_max - y_min);
                let workgroups_needed = pixel_count.div_ceil(descriptor.config.thread_count() as usize);

                if !batch.push_job(tile_id, workgroups_needed as u32, 1, 1, descriptor.config) {
                    break;
                }
            }
        }

        eprintln!(
            "compute: tile dispatch — {} tiles, {} total workgroups, kernel=0x{:08x}, scene=0x{:016x}",
            tiles_across * tiles_down,
            batch.jobs.len(),
            batch.kernel_tag,
            batch.scene_signature,
        );

        device.submit_batch(&batch)
    }

    /// Waits for all previously dispatched work to complete.
    pub fn wait_all_dispatched(&self) {
        self.active_device().wait_idle();
    }
}

fn hash32(bytes: &[u8]) -> u32 {
    let mut hash = 0x811c9dc5u32;
    for &byte in bytes {
        hash ^= byte as u32;
        hash = hash.wrapping_mul(0x01000193);
    }
    hash
}

/// SIMD capability snapshot used by diagnostic output.
pub struct SimdCapabilities {
    /// Preferred vector lanes for the current target.
    pub max_lanes: u32,
    /// AVX2 availability.
    pub has_avx2: bool,
    /// AVX availability.
    pub has_avx: bool,
    /// SSE2 availability.
    pub has_sse2: bool,
    /// SSE 4.2 availability.
    pub has_sse42: bool,
    /// NEON availability.
    pub has_neon: bool,
    /// SVE availability.
    pub has_sve: bool,
}

impl SimdCapabilities {
    /// Detects SIMD capabilities for the current target.
    pub fn detect() -> Self {
        let f = native_calls::host_detect_simd_features();
        let max_lanes = if f.avx2 || f.avx {
            8
        } else if f.sse2 || f.neon {
            4
        } else {
            1
        };

        Self {
            max_lanes,
            has_avx2: f.avx2,
            has_avx: f.avx,
            has_sse2: f.sse2,
            has_sse42: f.sse4_2,
            has_neon: f.neon,
            has_sve: f.sve,
        }
    }

    /// Prints a compact SIMD capability report.
    pub fn report(&self) {
        let mut features = Vec::new();
        if self.has_avx2 {
            features.push("AVX2");
        }
        if self.has_avx {
            features.push("AVX");
        }
        if self.has_sse42 {
            features.push("SSE4.2");
        }
        if self.has_sse2 {
            features.push("SSE2");
        }
        if self.has_neon {
            features.push("NEON");
        }
        if self.has_sve {
            features.push("SVE");
        }

        if features.is_empty() {
            eprintln!("simd: scalar-only (no SIMD extensions)");
        } else {
            eprintln!("simd: {} lanes — {}", self.max_lanes, features.join(" "));
        }
    }
}

/// Prints an extended compute environment diagnostic report.
pub fn diagnose_compute_environment() {
    use crate::core::engine::acces_hardware::CommandBuffer;
    
    eprintln!("\n╔══ Compute Environment Diagnosis ══════════════════════════════════╗");
    
    let simd_caps = SimdCapabilities::detect();
    simd_caps.report();
    
    let cpu_simd = GenericComputeDevice::new_cpu_simd();
    let cpu_scalar = GenericComputeDevice::new_cpu_scalar();
    
    let cpu_simd_caps = cpu_simd.capabilities();
    let cpu_scalar_caps = cpu_scalar.capabilities();
    
    eprintln!("cpu-simd:   {} lanes, {} max workgroup={}, shared={} bytes", 
        cpu_simd_caps.parallel_lanes, 
        cpu_simd_caps.max_workgroups,
        cpu_simd_caps.max_workgroup_size,
        cpu_simd_caps.shared_memory_bytes
    );
    eprintln!("cpu-scalar: {} lanes, {} max workgroup={}, shared={} bytes", 
        cpu_scalar_caps.parallel_lanes, 
        cpu_scalar_caps.max_workgroups,
        cpu_scalar_caps.max_workgroup_size,
        cpu_scalar_caps.shared_memory_bytes
    );
    
    let fake_gpu_caps = ComputeCapabilities::gpu(128, 512, 256, 65536);
    eprintln!("gpu-capability: {} lanes, {} max workgroup={}, shared={} bytes",
        fake_gpu_caps.parallel_lanes,
        fake_gpu_caps.max_workgroups,
        fake_gpu_caps.max_workgroup_size,
        fake_gpu_caps.shared_memory_bytes
    );
    
    let mut dispatcher = AdaptiveComputeDispatcher::new();
    dispatcher.register_device(Box::new(cpu_simd));
    dispatcher.register_device(Box::new(cpu_scalar));
    
    let native_backend = NativeHardwareBackend::detect();
    if let Some(gpu_backend) = native_backend.gpu_backend() {
        let gpu_device = GenericComputeDevice::new_gpu_with_fd(
            gpu_backend,
            gpu_backend.info().active_cu.max(4) * 8,
            1024,
            gpu_backend.info().active_cu.max(1),
            262144,
        );
        dispatcher.register_device(Box::new(gpu_device));

        eprintln!("gpu: mmap_active={} mmap_ptr={:?} mmap_len={} drm_fd={}",
            gpu_backend.is_mmap_active(),
            gpu_backend.mmap_framebuffer_ptr(),
            gpu_backend.mmap_framebuffer_len(),
            gpu_backend.drm_fd(),
        );
    }
    
    let devices = dispatcher.list_devices();
    eprintln!("\nregistered devices:");
    for device_info in devices {
        eprintln!("  {}", device_info);
    }
    
    let autodispatched = AdaptiveComputeDispatcher::with_auto_detection(native_backend.gpu_backend());
    eprintln!("\nauto-detected dispatcher: {} devices", autodispatched.device_count());
    
    let mut d2 = AdaptiveComputeDispatcher::new();
    d2.register_device(Box::new(GenericComputeDevice::new_cpu_scalar()));
    let device = d2.active_device_mut();
    let caps = device.capabilities();
    eprintln!("active_device_mut accessed: {} lanes", caps.parallel_lanes);
    
    let kernel_config = KernelConfig::new(8, 8, 1)
        .with_shared_memory(4096);
    
    eprintln!("\nkernel config:");
    eprintln!("  workgroup size: {}×{}×{}", 
        kernel_config.workgroup_size_x,
        kernel_config.workgroup_size_y,
        kernel_config.workgroup_size_z
    );
    eprintln!("  thread count per workgroup: {}", kernel_config.thread_count());
    eprintln!("  shared memory: {} bytes", kernel_config.shared_memory_bytes);
    
    let mut batch = ComputeJobBatch::new(256);
    let mut submitted_jobs = 0usize;
    for tile_id in 0..16 {
        if batch.push_job(tile_id, 4, 4, 1, kernel_config) {
            submitted_jobs = submitted_jobs.saturating_add(1);
        }
    }
    eprintln!("\njob batch:");
    eprintln!("  jobs submitted: {}", submitted_jobs);
    eprintln!("  job IDs: {} to {}", batch.jobs[0].job_id, batch.jobs[batch.jobs.len()-1].job_id);
    eprintln!("  total threads: {}", batch.total_threads());
    
    let queue = ComputeQueue::new();
    queue.submit_batch(batch.jobs.len() as u32);
    eprintln!("\nqueue status (before):");
    eprintln!("  pending jobs: {}", queue.pending_jobs());
    eprintln!("  is idle: {}", queue.is_idle());
    
    queue.mark_batch_complete(batch.jobs.len() as u32);
    eprintln!("queue status (after):");
    eprintln!("  pending jobs: {}", queue.pending_jobs());
    eprintln!("  is idle: {}", queue.is_idle());
    queue.wait_idle();
    eprintln!("  after wait_idle: still idle={}", queue.is_idle());
    
    let mut temp_batch = ComputeJobBatch::new(256);
    temp_batch.push_job(99, 1, 1, 1, kernel_config);
    temp_batch.clear();
    eprintln!("\nbatch after clear: {} jobs", temp_batch.jobs.len());
    
    let mut cmd_buf = CommandBuffer::new();
    cmd_buf.push_u32(0xdeadbeef);
    cmd_buf.push_u64(0x1234567890abcdef);
    cmd_buf.push_bytes(&[1, 2, 3, 4]);
    cmd_buf.align_to(8);
    eprintln!("\ncommand buffer: {} bytes, slice_len={}", cmd_buf.len(), cmd_buf.as_slice().len());
    cmd_buf.clear();
    eprintln!("command buffer after clear: {} bytes", cmd_buf.len());
    
    let active = dispatcher.active_device();
    _ = active.compile_kernel("test", b"void main() {}");
    _ = active.submit_batch(&batch);
    active.wait_idle();
    eprintln!("\nactive device called wait_idle");
    
    _ = dispatcher.set_active_device(if dispatcher.device_count() > 1 { 1 } else { 0 });
    _ = dispatcher.dispatch_tile_compute(640, 480, 16, kernel_config);
    dispatcher.wait_all_dispatched();
    
    if dispatcher.device_count() > 0 {
        let device_name = dispatcher.active_device().device_name();
        eprintln!("\nactive device: {}", device_name);
        let cap = dispatcher.active_device().capabilities();
        eprintln!("  kind: {:?}", cap.kind);
    }
    
    eprintln!("╚═══════════════════════════════════════════════════════════════════╝\n");
}


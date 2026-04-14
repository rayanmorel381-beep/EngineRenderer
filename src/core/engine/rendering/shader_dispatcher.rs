use crate::core::engine::acces_hardware::{
    ComputeCapabilities, ComputeDeviceKind, KernelConfig, ComputeJobBatch,
    ComputeQueue, GpuRenderBackend, GpuSubmitter,
};

pub trait ComputeDevice: Send + Sync {
    fn capabilities(&self) -> ComputeCapabilities;

    fn compile_kernel(
        &self,
        name: &str,
        kernel_source: &[u8],
    ) -> Result<Vec<u8>, String>;

    fn submit_batch(&self, batch: &ComputeJobBatch) -> Result<u64, String>;

    fn wait_idle(&self);

    fn device_name(&self) -> &str;
}

pub struct GenericComputeDevice {
    capabilities: ComputeCapabilities,
    queue: ComputeQueue,
    device_name: String,
    drm_fd: i32,
    driver_kind: ComputeDeviceKind,
    submitter: Option<std::sync::Mutex<GpuSubmitter>>,
}

impl GenericComputeDevice {
    pub fn new_gpu(
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
            queue: ComputeQueue::new(),
            device_name,
            drm_fd: -1,
            driver_kind: ComputeDeviceKind::Gpu,
            submitter: None,
        }
    }

    pub fn new_gpu_with_fd(
        backend: &GpuRenderBackend,
        max_workgroups: u32,
        max_workgroup_size: u32,
        parallel_lanes: u32,
        shared_memory_bytes: u32,
    ) -> Self {
        let mut dev = Self::new_gpu(backend, max_workgroups, max_workgroup_size, parallel_lanes, shared_memory_bytes);
        dev.drm_fd = backend.drm_fd();
        dev.submitter = Some(std::sync::Mutex::new(
            GpuSubmitter::new(backend.drm_fd(), backend.driver(), backend.gem_handle())
        ));
        dev
    }

    pub fn new_cpu_simd() -> Self {
        Self {
            capabilities: ComputeCapabilities::cpu_simd(),
            queue: ComputeQueue::new(),
            device_name: "CPU-SIMD".to_string(),
            drm_fd: -1,
            driver_kind: ComputeDeviceKind::CpuSimd,
            submitter: None,
        }
    }

    pub fn new_cpu_scalar() -> Self {
        Self {
            capabilities: ComputeCapabilities::cpu_scalar(),
            queue: ComputeQueue::new(),
            device_name: "CPU-Scalar".to_string(),
            drm_fd: -1,
            driver_kind: ComputeDeviceKind::CpuScalar,
            submitter: None,
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
        _kernel_source: &[u8],
    ) -> Result<Vec<u8>, String> {
        let device_type = match self.capabilities.kind {
            ComputeDeviceKind::Gpu => "GPU",
            ComputeDeviceKind::CpuSimd => "CPU-SIMD",
            ComputeDeviceKind::CpuScalar => "CPU-Scalar",
        };
        eprintln!(
            "compute: {} compiling kernel '{}' (device={})",
            self.device_name, name, device_type
        );

        Ok(Vec::new())
    }

    fn submit_batch(&self, batch: &ComputeJobBatch) -> Result<u64, String> {
        let batch_size = batch.jobs.len() as u32;
        let total_threads = batch.total_threads();

        self.queue.submit_batch(batch_size);

        if let Some(ref submitter_lock) = self.submitter {
            let mut submitter = submitter_lock.lock().unwrap();
            let total_tiles = batch.jobs.len() as u32;
            let workgroup_size = batch.jobs.first()
                .map(|j| j.config.thread_count())
                .unwrap_or(256);
            let pixel_count = total_threads as u32;

            submitter.build_compute_dispatch(total_tiles, workgroup_size, pixel_count);

            match submitter.submit() {
                Ok(cs_id) => {
                    eprintln!(
                        "gpu-compute: {} dispatched {} tiles ({} threads) → cs_id={} driver={} dwords={}",
                        self.device_name, total_tiles, total_threads, cs_id,
                        submitter.driver().name(), submitter.cmd_buf_dwords(),
                    );
                }
                Err(e) => {
                    eprintln!(
                        "gpu-compute: {} DRM submit failed ({}), CPU fallback",
                        self.device_name, e,
                    );
                }
            }
        } else {
            let dispatch_target = match self.driver_kind {
                ComputeDeviceKind::Gpu => "gpu-sw",
                ComputeDeviceKind::CpuSimd => "cpu-simd",
                ComputeDeviceKind::CpuScalar => "cpu-scalar",
            };
            eprintln!(
                "compute: {} submitted batch {} jobs × {} total threads (target={}, fd={})",
                self.device_name, batch_size, total_threads, dispatch_target, self.drm_fd
            );
        }

        self.queue.mark_batch_complete(batch_size);
        Ok(total_threads)
    }

    fn wait_idle(&self) {
        self.queue.wait_idle();
    }

    fn device_name(&self) -> &str {
        &self.device_name
    }
}

pub struct AdaptiveComputeDispatcher {
    devices: Vec<Box<dyn ComputeDevice>>,
    active_device_idx: usize,
}

impl AdaptiveComputeDispatcher {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            active_device_idx: 0,
        }
    }

    pub fn register_device(&mut self, device: Box<dyn ComputeDevice>) {
        eprintln!("compute: registered device '{}' — {}", device.device_name(), self.device_count() + 1);
        self.devices.push(device);
    }

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

        let has_simd = {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                std::is_x86_feature_detected!("avx2") || std::is_x86_feature_detected!("sse2")
            }
            #[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
            {
                true
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "arm", target_arch = "aarch64")))]
            {
                false
            }
        };

        let cpu_device = if has_simd {
            Box::new(GenericComputeDevice::new_cpu_simd()) as Box<dyn ComputeDevice>
        } else {
            Box::new(GenericComputeDevice::new_cpu_scalar()) as Box<dyn ComputeDevice>
        };
        dispatcher.register_device(cpu_device);

        dispatcher
    }

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

    pub fn active_device(&self) -> &dyn ComputeDevice {
        &*self.devices[self.active_device_idx]
    }

    pub fn active_device_mut(&mut self) -> &mut dyn ComputeDevice {
        &mut *self.devices[self.active_device_idx]
    }

    pub fn device_count(&self) -> usize {
        self.devices.len()
    }

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

    pub fn dispatch_tile_compute(
        &mut self,
        image_width: usize,
        image_height: usize,
        tile_size: usize,
        config: KernelConfig,
    ) -> Result<u64, String> {
        let device = self.active_device();
        let mut batch = ComputeJobBatch::new(device.capabilities().max_workgroups as usize);

        let tiles_across = image_width.div_ceil(tile_size);
        let tiles_down = image_height.div_ceil(tile_size);

        for tile_y in 0..tiles_down {
            for tile_x in 0..tiles_across {
                let tile_id = (tile_y * tiles_across + tile_x) as u32;
                let x_min = tile_x * tile_size;
                let y_min = tile_y * tile_size;
                let x_max = (x_min + tile_size).min(image_width);
                let y_max = (y_min + tile_size).min(image_height);

                let pixel_count = (x_max - x_min) * (y_max - y_min);
                let workgroups_needed = pixel_count.div_ceil(config.thread_count() as usize);

                if !batch.push_job(tile_id, workgroups_needed as u32, 1, 1, config) {
                    break;
                }
            }
        }

        eprintln!(
            "compute: tile dispatch — {} tiles, {} total workgroups",
            tiles_across * tiles_down,
            batch.jobs.len()
        );

        device.submit_batch(&batch)
    }

    pub fn wait_all_dispatched(&self) {
        self.active_device().wait_idle();
    }
}

pub struct SimdCapabilities {
    pub max_lanes: u32,
    pub has_avx2: bool,
    pub has_avx: bool,
    pub has_sse2: bool,
    pub has_sse42: bool,
    pub has_neon: bool,
    pub has_sve: bool,
}

impl SimdCapabilities {
    pub fn detect() -> Self {
        let (has_avx2, has_avx, has_sse2, has_sse42) = {
            #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
            {
                (
                    std::is_x86_feature_detected!("avx2"),
                    std::is_x86_feature_detected!("avx"),
                    std::is_x86_feature_detected!("sse2"),
                    std::is_x86_feature_detected!("sse4.2"),
                )
            }
            #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
            {
                (false, false, false, false)
            }
        };

        let (has_neon, has_sve) = if cfg!(target_arch = "aarch64") {
            (cfg!(target_feature = "neon"), cfg!(target_feature = "sve"))
        } else {
            (false, false)
        };

        let max_lanes = if has_avx2 || has_avx {
            8
        } else if has_sse2 || has_neon {
            4
        } else {
            1
        };

        Self {
            max_lanes,
            has_avx2,
            has_avx,
            has_sse2,
            has_sse42,
            has_neon,
            has_sve,
        }
    }

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
    
    if let Some(gpu_backend) = crate::core::engine::acces_hardware::GpuRenderBackend::try_init() {
        let gpu_device = GenericComputeDevice::new_gpu_with_fd(
            &gpu_backend,
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
    
    let _devices = dispatcher.list_devices();
    eprintln!("\nregistered devices:");
    for device_info in _devices {
        eprintln!("  {}", device_info);
    }
    
    let autodispatched = AdaptiveComputeDispatcher::with_auto_detection(None);
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
    for tile_id in 0..16 {
        let _ = batch.push_job(tile_id, 4, 4, 1, kernel_config);
    }
    eprintln!("\njob batch:");
    eprintln!("  jobs submitted: {}", batch.jobs.len());
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


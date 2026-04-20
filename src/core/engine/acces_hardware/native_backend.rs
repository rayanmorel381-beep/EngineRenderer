use std::{collections::HashMap, path::PathBuf, sync::Mutex};

use crate::core::engine::acces_hardware::{
    arch::compute_dispatch, ComputeCapabilities, ComputeDispatchMetadata, ComputeDeviceKind,
    ComputeJobBatch, ComputeQueue, CpuProfile, GpuRenderBackend, GpuSubmitter,
    HardwareCapabilities,
};

#[derive(Debug, Clone, Copy)]
pub struct RamRuntimeConfig {
    pub page_size: usize,
    pub total_bytes: u64,
    pub available_bytes: Option<u64>,
}

#[derive(Debug)]
pub struct NativeHardwareBackend {
    hw_caps: HardwareCapabilities,
    cpu_profile: CpuProfile,
    ram_config: RamRuntimeConfig,
    gpu_backend: Option<GpuRenderBackend>,
}

impl NativeHardwareBackend {
    pub fn detect() -> Self {
        let hw_caps = HardwareCapabilities::detect();
        let cpu_profile = CpuProfile::detect();
        let ram = compute_dispatch::default_ram_config();
        let ram_config = RamRuntimeConfig {
            page_size: ram.page_size,
            total_bytes: ram.total_bytes,
            available_bytes: ram.available_bytes,
        };
        let gpu_backend = GpuRenderBackend::try_init();

        Self {
            hw_caps,
            cpu_profile,
            ram_config,
            gpu_backend,
        }
    }

    pub fn hw_caps(&self) -> &HardwareCapabilities {
        &self.hw_caps
    }

    pub fn cpu_profile(&self) -> &CpuProfile {
        &self.cpu_profile
    }

    pub fn ram_config(&self) -> RamRuntimeConfig {
        self.ram_config
    }

    pub fn gpu_backend(&self) -> Option<&GpuRenderBackend> {
        self.gpu_backend.as_ref()
    }

    pub fn probe_gpu_backend(&self) -> Option<GpuRenderBackend> {
        if self.gpu_backend.is_some() {
            GpuRenderBackend::try_init()
        } else {
            None
        }
    }

    pub fn capabilities_for(&self, kind: ComputeDeviceKind) -> ComputeCapabilities {
        match kind {
            ComputeDeviceKind::Gpu => {
                if let Some(gpu) = self.gpu_backend.as_ref() {
                    ComputeCapabilities::gpu(
                        gpu.info().active_cu.saturating_mul(4).max(1),
                        256,
                        gpu.compute_units().max(1),
                        65_536,
                    )
                } else {
                    ComputeCapabilities::cpu_simd()
                }
            }
            ComputeDeviceKind::CpuSimd => ComputeCapabilities::cpu_simd(),
            ComputeDeviceKind::CpuScalar => ComputeCapabilities::cpu_scalar(),
        }
    }

    pub fn create_compute_backend(&self, requested: ComputeDeviceKind) -> NativeComputeBackend {
        match requested {
            ComputeDeviceKind::Gpu => {
                if let Some(gpu) = self.gpu_backend.as_ref() {
                    let lanes = gpu.compute_units().max(1);
                    NativeComputeBackend::new(
                        format!("GPU-{}-{}cu", gpu.driver_name(), gpu.compute_units()),
                        ComputeDeviceKind::Gpu,
                        lanes,
                        gpu.drm_fd(),
                        Some(GpuSubmitter::new(gpu.drm_fd(), gpu.driver(), gpu.gem_handle())),
                    )
                } else {
                    self.create_compute_backend(ComputeDeviceKind::CpuSimd)
                }
            }
            ComputeDeviceKind::CpuSimd => {
                let lanes = ComputeCapabilities::cpu_simd().parallel_lanes;
                NativeComputeBackend::new(
                    "CPU-SIMD".to_string(),
                    ComputeDeviceKind::CpuSimd,
                    lanes,
                    -1,
                    None,
                )
            }
            ComputeDeviceKind::CpuScalar => NativeComputeBackend::new(
                "CPU-Scalar".to_string(),
                ComputeDeviceKind::CpuScalar,
                1,
                -1,
                None,
            ),
        }
    }

    pub fn into_gpu_backend(self) -> Option<GpuRenderBackend> {
        self.gpu_backend
    }
}

pub struct NativeComputeBackend {
    device_name: String,
    driver_kind: ComputeDeviceKind,
    parallel_lanes: u32,
    drm_fd: i32,
    queue: ComputeQueue,
    submitter: Option<Mutex<GpuSubmitter>>,
    kernel_cache: Mutex<HashMap<String, Vec<u8>>>,
}

impl NativeComputeBackend {
    pub fn new(
        device_name: String,
        driver_kind: ComputeDeviceKind,
        parallel_lanes: u32,
        drm_fd: i32,
        submitter: Option<GpuSubmitter>,
    ) -> Self {
        Self {
            device_name,
            driver_kind,
            parallel_lanes,
            drm_fd,
            queue: ComputeQueue::new(),
            submitter: submitter.map(Mutex::new),
            kernel_cache: Mutex::new(HashMap::new()),
        }
    }

    pub fn compile_kernel(&self, name: &str, kernel_source: &[u8]) -> Result<Vec<u8>, String> {
        let key = format!("{}-{:08x}", name, hash32(kernel_source));
        if let Some(binary) = lock_unpoisoned(&self.kernel_cache).get(&key).cloned() {
            crate::runtime_log!(
                "compute: {} kernel '{}' cache-hit size={}B",
                self.device_name,
                name,
                binary.len(),
            );
            return Ok(binary);
        }

        let cache_path = self.persistent_kernel_cache_path(name, kernel_source);
        if let Ok(binary) = std::fs::read(&cache_path) {
            crate::runtime_log!(
                "compute: {} kernel '{}' disk-cache-hit size={}B",
                self.device_name,
                name,
                binary.len(),
            );
            lock_unpoisoned(&self.kernel_cache).insert(key, binary.clone());
            return Ok(binary);
        }

        crate::runtime_log!(
            "compute: {} compiling kernel '{}' (device={:?})",
            self.device_name,
            name,
            self.driver_kind,
        );

        let mut binary = Vec::with_capacity(kernel_source.len() + 32);
        binary.extend_from_slice(b"ERKERN1");
        binary.push(match self.driver_kind {
            ComputeDeviceKind::Gpu => 0,
            ComputeDeviceKind::CpuSimd => 1,
            ComputeDeviceKind::CpuScalar => 2,
        });
        binary.extend_from_slice(&self.parallel_lanes.to_le_bytes());
        binary.extend_from_slice(&(kernel_source.len() as u32).to_le_bytes());
        binary.extend_from_slice(&hash32(kernel_source).to_le_bytes());
        binary.extend_from_slice(kernel_source);

        if let Some(parent) = cache_path.parent()
            && !parent.as_os_str().is_empty()
            && let Err(error) = std::fs::create_dir_all(parent)
        {
            crate::runtime_log!("compute: kernel cache dir create failed: {}", error);
        }
        if let Err(error) = std::fs::write(&cache_path, &binary) {
            crate::runtime_log!("compute: kernel disk-cache store failed: {}", error);
        }

        lock_unpoisoned(&self.kernel_cache).insert(key, binary.clone());
        Ok(binary)
    }

    pub fn submit_batch(&self, batch: &ComputeJobBatch) -> Result<u64, String> {
        let batch_size = batch.jobs.len() as u32;
        let total_threads = batch.total_threads();
        self.queue.submit_batch(batch_size);

        if let Some(ref submitter_lock) = self.submitter {
            let mut submitter = lock_unpoisoned(submitter_lock);
            let total_tiles = batch.jobs.len() as u32;
            let workgroup_size = batch.jobs.first().map(|job| job.config.thread_count()).unwrap_or(256);
            let pixel_count = total_threads as u32;

            submitter.build_compute_dispatch_with_metadata(
                total_tiles,
                workgroup_size,
                pixel_count,
                ComputeDispatchMetadata {
                    kernel_tag: batch.kernel_tag,
                    kernel_size_bytes: batch.kernel_size_bytes,
                    scene_signature: batch.scene_signature,
                    object_count: batch.object_count,
                    triangle_count: batch.triangle_count,
                },
            );

            match submitter.submit() {
                Ok(cs_id) => {
                    crate::runtime_log!(
                        "gpu-compute: {} dispatched {} tiles ({} threads) kernel=0x{:08x} scene=0x{:016x} objs={} tris={} → cs_id={} driver={} dwords={}",
                        self.device_name,
                        total_tiles,
                        total_threads,
                        batch.kernel_tag,
                        batch.scene_signature,
                        batch.object_count,
                        batch.triangle_count,
                        cs_id,
                        submitter.driver().name(),
                        submitter.cmd_buf_dwords(),
                    );
                }
                Err(error) => {
                    crate::runtime_log!(
                        "gpu-compute: {} DRM submit failed ({}), CPU fallback",
                        self.device_name,
                        error,
                    );
                }
            }
        } else {
            let dispatch_target = match self.driver_kind {
                ComputeDeviceKind::Gpu => "gpu-sw",
                ComputeDeviceKind::CpuSimd => "cpu-simd",
                ComputeDeviceKind::CpuScalar => "cpu-scalar",
            };
            crate::runtime_log!(
                "compute: {} submitted batch {} jobs × {} total threads (target={}, fd={})",
                self.device_name,
                batch_size,
                total_threads,
                dispatch_target,
                self.drm_fd,
            );
        }

        self.queue.mark_batch_complete(batch_size);
        Ok(total_threads)
    }

    pub fn wait_idle(&self) {
        self.queue.wait_idle();
    }

    pub fn device_name(&self) -> &str {
        &self.device_name
    }

    fn persistent_kernel_cache_path(&self, name: &str, kernel_source: &[u8]) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push("enginerenderer-cache");
        path.push("kernel-cache");
        path.push(self.device_name.replace('/', "_"));
        path.push(format!("{}-{:08x}.bin", name, hash32(kernel_source)));
        path
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

fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

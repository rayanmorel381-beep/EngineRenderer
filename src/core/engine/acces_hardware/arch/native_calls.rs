use crate::core::engine::acces_hardware::cpu::CpuProfile;
use crate::core::engine::acces_hardware::gpu::{GpuRenderBackend, gpu_dispatch_tiles};

#[derive(Debug, Clone, Copy)]
pub struct NativeCpuCall {
    pub architecture: &'static str,
    pub logical_cores: u8,
    pub vector_width_bits: u32,
}

pub fn native_cpu_call(cpu: &CpuProfile) -> NativeCpuCall {
    let architecture = if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else {
        "other"
    };

    NativeCpuCall {
        architecture,
        logical_cores: cpu.logical_cores,
        vector_width_bits: cpu.vector_width_bits,
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NativeGpuCall {
    pub init_ok: bool,
    pub dispatch_ok: bool,
    pub framebuffer_ok: bool,
}

pub fn native_gpu_call(gpu: Option<&GpuRenderBackend>, workgroup_size: usize) -> NativeGpuCall {
    let dispatched = gpu_dispatch_tiles(1, workgroup_size.max(1) as u32);
    let init_ok = gpu.is_some();
    let framebuffer_ok = gpu.map(|g| g.has_active_framebuffer()).unwrap_or(false);

    NativeGpuCall {
        init_ok,
        dispatch_ok: dispatched > 0,
        framebuffer_ok,
    }
}

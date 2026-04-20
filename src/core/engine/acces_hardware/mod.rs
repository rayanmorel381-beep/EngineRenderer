
//! Hardware access layer exports used by rendering and diagnostics.

pub(crate) mod arch;
pub mod native_backend;

pub mod cpu;
pub mod gpu;
pub mod ram;
pub mod display;
pub mod dma;
pub mod timer;

pub use arch::capabilities::HardwareCapabilities;
pub use arch::compute_dispatch::{
    CommandBuffer, ComputeCapabilities, ComputeDeviceKind, ComputeJobBatch, ComputeQueue, KernelConfig,
};
pub use native_backend::{NativeComputeBackend, NativeHardwareBackend, RamRuntimeConfig};
pub use cpu::{CpuProfile, pin_thread_to_core};
pub use gpu::{ComputeDispatchMetadata, DrmDriver, GpuRenderBackend, GpuSubmitter, arch_optimal_workgroup, gpu_dispatch_tiles};
pub use display::NativeWindow;
pub use dma::{DmaFramebuffer, alloc_dma_framebuffer};
pub use arch::native_calls::{native_cpu_call, native_gpu_call};
pub use timer::{HwInstant, elapsed_ms, precise_timestamp_ns};

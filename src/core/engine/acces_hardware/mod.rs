//! Hardware abstraction layer — bridges the `hardware` crate into the
//! rendering engine for GPU compute, memory management, CPU topology,
//! and high-precision timing.

pub mod capabilities;
pub mod cpu;
pub mod dma;
pub mod gpu;
pub mod gpu_render;
pub mod timer;

pub use capabilities::HardwareCapabilities;
pub use cpu::{CpuProfile, pin_thread_to_core};
pub use dma::{alloc_dma_framebuffer, DmaFramebuffer};
pub use gpu::gpu_dispatch_tiles;
pub use gpu_render::GpuRenderBackend;
pub use timer::{elapsed_ms, precise_timestamp_ns, HwInstant};

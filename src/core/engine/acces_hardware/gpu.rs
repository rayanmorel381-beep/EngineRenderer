//! GPU compute kernel registration and dispatch.

use hardware::sys;

/// Submits a compute workload to the GPU command queue if available.
///
/// Returns `true` if the command was enqueued. In the current hardware
/// crate this tracks dispatch counts and kernel invocations; actual GPU
/// execution requires MMIO-mapped queue submission which the kernel
/// handles transparently when DRM is available.
pub fn gpu_dispatch_tiles(tile_count: usize, workgroup_size: usize) -> bool {
    let kernel = match sys::gpu::compute::kernel::register_kernel(0, workgroup_size) {
        Some(k) => k,
        None => return false,
    };
    sys::gpu::compute::kernel::dispatch(&kernel, tile_count);
    true
}

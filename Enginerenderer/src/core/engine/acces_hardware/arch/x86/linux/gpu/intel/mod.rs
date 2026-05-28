//! Intel GPU bindings — `i915` DRM userspace driver (Gen3+ through Xe-LP).

pub(super) mod backend;
pub(super) mod scheduler;

mod drm_ffi;
mod i915;

pub(crate) use i915::{
    drm_i915_alloc_gem, drm_i915_gem_mmap_gtt, drm_i915_gem_wait,
    probe_i915_telemetry, submit_i915_execbuf,
};

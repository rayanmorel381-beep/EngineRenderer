//! AMD GPU bindings — legacy `radeon` and modern `amdgpu` DRM userspace drivers.

pub(super) mod backend;
pub(super) mod scheduler;

mod amdgpu;
mod drm_ffi;
mod radeon;

pub(crate) use amdgpu::{
    drm_amdgpu_alloc_gem, drm_amdgpu_gem_mmap, drm_amdgpu_wait_cs, probe_amdgpu_telemetry,
    submit_amdgpu_cs,
};
pub(crate) use radeon::{
    drm_radeon_alloc_gem, drm_radeon_gem_mmap, drm_radeon_gem_wait, probe_radeon_telemetry,
    submit_radeon_cs,
};

#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub(super) mod arm;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(super) mod x86;
pub(crate) mod compute_dispatch;
pub(crate) mod native_calls;
pub(crate) mod capabilities;


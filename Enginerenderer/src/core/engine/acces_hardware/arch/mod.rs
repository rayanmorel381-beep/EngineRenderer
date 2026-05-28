#[cfg(any(target_arch = "arm", target_arch = "aarch64"))]
pub(super) mod arm;
pub(crate) mod capabilities;
pub(crate) mod compute_dispatch;
pub(crate) mod native_calls;
#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
pub(super) mod x86;

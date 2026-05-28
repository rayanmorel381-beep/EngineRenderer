//! Apple/ARM64 CPU detection on Windows via Win32 registry + GetSystemInfo.

pub(super) mod backend;
pub(super) mod scheduler;
mod detect;
mod registry;

pub(crate) use detect::detect_arm;

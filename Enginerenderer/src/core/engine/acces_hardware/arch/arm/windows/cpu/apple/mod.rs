//! Apple/ARM64 CPU detection on Windows via Win32 registry + GetSystemInfo.

pub(super) mod backend;
mod detect;
mod registry;
pub(super) mod scheduler;

pub(crate) use detect::detect_arm;

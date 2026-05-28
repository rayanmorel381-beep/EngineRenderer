//! Intel CPU detection on Windows via the Win32 registry.

pub(super) mod backend;
mod detect;
mod registry;
pub(super) mod scheduler;

pub(crate) use detect::detect_intel;

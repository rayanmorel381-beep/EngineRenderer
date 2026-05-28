//! Intel CPU detection on Windows via the Win32 registry.

pub(super) mod backend;
pub(super) mod scheduler;
mod detect;
mod registry;

pub(crate) use detect::detect_intel;

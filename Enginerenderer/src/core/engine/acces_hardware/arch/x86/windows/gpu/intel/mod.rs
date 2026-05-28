//! Intel GPU detection on Windows via PCI adapter list + display-class registry.

pub(super) mod backend;
pub(super) mod scheduler;
mod probe;
mod registry;
mod types;

pub(crate) use probe::probe;
pub(crate) use types::GpuProbeResult;

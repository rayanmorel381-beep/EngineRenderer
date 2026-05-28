//! Intel GPU detection on Windows via PCI adapter list + display-class registry.

pub(super) mod backend;
mod probe;
mod registry;
pub(super) mod scheduler;
mod types;

pub(crate) use probe::probe;
pub(crate) use types::GpuProbeResult;

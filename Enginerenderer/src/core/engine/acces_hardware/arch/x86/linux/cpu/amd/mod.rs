//! AMD CPU detection on Linux via /proc/cpuinfo and cpufreq sysfs.

pub(super) mod backend;
mod cpuinfo;
mod detect;
pub(super) mod scheduler;

pub(crate) use detect::detect_amd;

//! Intel CPU detection on Linux via /proc/cpuinfo and cpufreq sysfs.

pub(super) mod backend;
pub(super) mod scheduler;
mod cpuinfo;
mod detect;

pub(crate) use detect::detect_intel;

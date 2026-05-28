//! Apple Silicon CPU detection on macOS via `sysctlbyname`.

pub(super) mod backend;
mod detect;
pub(super) mod scheduler;
mod sysctl;

pub(crate) use detect::{AppleSiliconInfo, detect_apple_silicon};

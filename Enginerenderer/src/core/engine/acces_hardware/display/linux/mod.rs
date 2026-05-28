//! Linux display server bindings (X11 + GLX).
//!
//! OS-specific, vendor-agnostic, architecture-agnostic. The same code runs on
//! `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, and any other
//! Linux target with libX11.so + libGL.so available at runtime.

pub(crate) mod glx;
pub(crate) mod x11;
mod screen;

pub use screen::LinuxWindow;

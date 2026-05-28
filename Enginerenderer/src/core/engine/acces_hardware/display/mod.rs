//! Native display layer.
//!
//! Provides [`NativeWindow`], an OS-agnostic façade that opens a real window
//! with an OpenGL 3.3 core context attached. The implementation is dispatched
//! at compile time via `cfg(target_os = ...)` to a backend module:
//!
//! - **Linux** → [`linux`] (X11 + GLX)
//! - **Windows** → [`windows`] (planned, currently stub)
//! - **macOS** → [`macos`] (planned, currently stub)
//! - **Android** → [`android`] (EGL + GLES3 pbuffer)
//!
//! Vendor-specific tuning (vsync slot count, latency budget, target FPS) is
//! pulled from [`crate::core::engine::acces_hardware::arch`] which is keyed
//! by `(architecture, os, vendor)`. Window-system code itself is OS-only —
//! the X11/GLX bindings work identically on AMD, Intel, Mesa, and Apple Mesa
//! drivers under Linux, regardless of CPU architecture.

mod backend;
mod event;
mod screen;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "android")]
pub mod android;

pub(crate) use backend::WindowBackend;
pub use event::BackendEvent;
pub use screen::NativeWindow;

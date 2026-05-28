//! Public façade for the native display layer.
//!
//! Exposes [`NativeWindow`] so external binaries / examples can open a real
//! OS window with an OpenGL 3.3 core context. On platforms without a backend
//! implementation, `NativeWindow::open` still returns a logical window and
//! `has_backend()` returns `false`.

pub use crate::core::engine::acces_hardware::display::{BackendEvent, NativeWindow};

#[cfg(target_os = "linux")]
pub use linux_offscreen::{DesktopOffscreenContext, desktop_offscreen_context};

#[cfg(target_os = "linux")]
mod linux_offscreen {
    use core::ffi::c_void;

    use crate::core::engine::acces_hardware::display::linux::glx::{
        OffscreenContext, create_offscreen_context,
    };

    /// Public, opaque wrapper around a GLX offscreen Pbuffer context for
    /// headless GPU compute on Linux desktops.
    pub struct DesktopOffscreenContext {
        inner: OffscreenContext,
    }

    impl DesktopOffscreenContext {
        /// Returns the Pbuffer width in pixels.
        pub fn width(&self) -> u32 {
            self.inner.width
        }

        /// Returns the Pbuffer height in pixels.
        pub fn height(&self) -> u32 {
            self.inner.height
        }

        /// Binds this context to the calling thread.
        pub fn make_current(&self) -> bool {
            self.inner.make_current()
        }

        /// Resolves a GL function pointer by null-terminated name.
        pub fn gl_get_proc(&self, name: &[u8]) -> *mut c_void {
            self.inner.get_proc(name)
        }
    }

    /// Creates a fresh GLX offscreen context bound to a Pbuffer of the given
    /// size, requesting an OpenGL `major.minor` core profile context.
    pub fn desktop_offscreen_context(
        width: u32,
        height: u32,
        major: i32,
        minor: i32,
    ) -> Option<DesktopOffscreenContext> {
        create_offscreen_context(width, height, major, minor)
            .map(|inner| DesktopOffscreenContext { inner })
    }
}

//! Windows display backend stub.
//!
//! Will host Win32 window creation + WGL OpenGL context bindings. Currently
//! returns `None` so callers fall back to the software path.

mod screen;

pub use screen::WindowsWindow;

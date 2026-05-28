//! macOS display backend stub.
//!
//! Will host Cocoa NSWindow + NSOpenGL context bindings. Currently returns
//! `None` so callers fall back to the software path.

mod screen;

pub use screen::MacosWindow;

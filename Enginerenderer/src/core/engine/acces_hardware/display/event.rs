//! Backend-agnostic window event surfaced to [`super::NativeWindow`].

/// Backend-agnostic event surfaced to [`super::NativeWindow::pump`].
#[derive(Debug, Clone, Copy)]
pub enum BackendEvent {
	CloseRequested,
	Resized { width: u32, height: u32 },
	Expose,
	KeyPress { keysym: u64 },
	KeyRelease { keysym: u64 },
	MouseButtonPress { button: u32, x: i32, y: i32 },
	MouseButtonRelease { button: u32, x: i32, y: i32 },
	MouseMove { x: i32, y: i32 },
}

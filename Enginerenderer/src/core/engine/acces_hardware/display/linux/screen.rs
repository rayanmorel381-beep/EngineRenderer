//! `LinuxWindow` — composite of an X11 window + GLX context implementing
//! [`WindowBackend`].

use super::super::{BackendEvent, WindowBackend};
use super::glx::{GlxContext, choose_visual_only, create_context};
use super::x11::{X11Event, X11Window, xlib};

/// Opaque window/context pair owning all native resources.
pub struct LinuxWindow {
	window: X11Window,
	context: GlxContext,
}

impl LinuxWindow {
	fn open_impl(width: u32, height: u32, title: &str) -> Option<Self> {
		let lib = xlib()?;
		let display = unsafe { (lib.XOpenDisplay)(core::ptr::null()) };
		if display.is_null() {
			return None;
		}
		let screen = unsafe { (lib.XDefaultScreen)(display) };
		let (visual_info, fb_config) = match choose_visual_only(display, screen) {
			Some(v) => v,
			None => {
				unsafe { (lib.XCloseDisplay)(display) };
				return None;
			}
		};

		let window = X11Window::create_on_display(
			display,
			true,
			width,
			height,
			title,
			Some(copy_visual_info(&visual_info)),
		)?;
		let context = create_context(
			window.display,
			fb_config,
			window.window,
			copy_visual_info(&visual_info),
		)?;
		context.set_swap_interval(0);
		Some(Self { window, context })
	}
}

impl WindowBackend for LinuxWindow {
	fn open(width: u32, height: u32, title: &str) -> Option<Self> {
		Self::open_impl(width, height, title)
	}

	fn width(&self) -> u32 {
		self.window.width
	}

	fn height(&self) -> u32 {
		self.window.height
	}

	fn should_close(&self) -> bool {
		self.window.closed
	}

	fn pump_events(&mut self) -> Vec<BackendEvent> {
		self.window
			.pump_events()
			.into_iter()
			.map(convert_event)
			.collect()
	}

	fn make_current(&self) -> bool {
		self.context.make_current()
	}

	fn swap_buffers(&self) {
		self.context.swap_buffers();
	}

	fn get_proc_address(&self, name: &[u8]) -> *mut core::ffi::c_void {
		self.context.get_proc(name)
	}

	fn take_dropped_files(&mut self) -> Vec<String> {
		self.window.take_dropped_files()
	}
}

#[allow(clippy::unnecessary_cast)]
fn convert_event(ev: X11Event) -> BackendEvent {
	match ev {
		X11Event::CloseRequested => BackendEvent::CloseRequested,
		X11Event::Resized { width, height } => BackendEvent::Resized { width, height },
		X11Event::Expose => BackendEvent::Expose,
		X11Event::KeyPress { keysym } => BackendEvent::KeyPress {
			keysym: keysym as u64,
		},
		X11Event::KeyRelease { keysym } => BackendEvent::KeyRelease {
			keysym: keysym as u64,
		},
		X11Event::MouseButtonPress { button, x, y } => BackendEvent::MouseButtonPress {
			button: button as u32,
			x: x as i32,
			y: y as i32,
		},
		X11Event::MouseButtonRelease { button, x, y } => BackendEvent::MouseButtonRelease {
			button: button as u32,
			x: x as i32,
			y: y as i32,
		},
		X11Event::MouseMove { x, y } => BackendEvent::MouseMove {
			x: x as i32,
			y: y as i32,
		},
	}
}

fn copy_visual_info(src: &super::x11::XVisualInfo) -> super::x11::XVisualInfo {
	super::x11::XVisualInfo {
		visual: src.visual,
		visualid: src.visualid,
		screen: src.screen,
		depth: src.depth,
		class: src.class,
		red_mask: src.red_mask,
		green_mask: src.green_mask,
		blue_mask: src.blue_mask,
		colormap_size: src.colormap_size,
		bits_per_rgb: src.bits_per_rgb,
	}
}

//! [`NativeWindow`] OS-agnostic façade with cached GL proc resolution.

use std::sync::Mutex;

use super::backend::{Backend, WindowBackend};
use super::event::BackendEvent;
use crate::core::engine::acces_hardware::arch::compute_dispatch;

type ProcCacheEntry = (Box<[u8]>, *mut core::ffi::c_void);

/// Native window handle with optional OS backend, GPU context attached on
/// supported platforms.
pub struct NativeWindow {
	pub width: u32,
	pub height: u32,
	pub vsync_capacity: usize,
	backend: Option<Backend>,
	closed: bool,
	proc_cache: Mutex<Vec<ProcCacheEntry>>,
}

impl core::fmt::Debug for NativeWindow {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.debug_struct("NativeWindow")
			.field("width", &self.width)
			.field("height", &self.height)
			.field("vsync_capacity", &self.vsync_capacity)
			.field("has_backend", &self.backend.is_some())
			.field("closed", &self.closed)
			.finish()
	}
}

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

impl NativeWindow {
	/// Opens a native window at the requested logical size. On platforms where
	/// the backend is implemented this creates a real OS window and an OpenGL
	/// context. On unsupported platforms the window is a logical placeholder
	/// (the previous behaviour is preserved).
	pub fn open(width: usize, height: usize, title: &str) -> Option<Self> {
		let cfg = compute_dispatch::default_display_config();
		let schedule = compute_dispatch::build_display_schedule(width.saturating_mul(height));
		let title_scale = title.chars().count().clamp(1, 64);
		let vendor_scale = match cfg.vendor {
			compute_dispatch::Vendor::Amd => 1usize,
			compute_dispatch::Vendor::Intel => 1usize,
			compute_dispatch::Vendor::Apple => 1usize,
			compute_dispatch::Vendor::Unknown => 1usize,
		};
		let buffering_scale = if cfg.double_buffered { 2usize } else { 1usize };
		let latency_scale = (cfg.latency_budget_us.max(1) as usize)
			.saturating_div(cfg.scan_out_latency_us.max(1) as usize)
			.max(1);
		let fps_scale = (cfg.target_render_fps.max(1) as usize)
			.saturating_div(60)
			.max(1);
		let vsync_capacity = compute_dispatch::clamp_display_workers(
			cfg.vsync_slots
				.max(schedule.chunks)
				.saturating_add(title_scale.saturating_div(32))
				.saturating_mul(vendor_scale)
				.saturating_mul(buffering_scale)
				.saturating_mul(latency_scale)
				.saturating_mul(fps_scale)
				.max(1),
		);
		let width_px = width.max(cfg.page_size / 1024).max(1) as u32;
		let height_px = height.max(cfg.page_size / 1024).max(1) as u32;
		let backend = Backend::open(width_px, height_px, title);
		let (final_w, final_h) = match backend.as_ref() {
			Some(b) => (b.width(), b.height()),
			None => (width_px, height_px),
		};
		Some(Self {
			width: final_w,
			height: final_h,
			vsync_capacity,
			backend,
			closed: false,
			proc_cache: Mutex::new(Vec::with_capacity(64)),
		})
	}

	pub fn should_close(&self) -> bool {
		if let Some(b) = self.backend.as_ref() {
			return self.closed || b.should_close();
		}
		self.closed
	}

	/// Pumps OS events (close, resize, input). Returns the events that were
	/// processed; updates internal width/height when the window was resized.
	pub fn pump(&mut self) -> Vec<BackendEvent> {
		let Some(b) = self.backend.as_mut() else {
			return Vec::new();
		};
		let events = b.pump_events();
		for ev in &events {
			match ev {
				BackendEvent::CloseRequested => self.closed = true,
				BackendEvent::Resized { width, height } => {
					self.width = *width;
					self.height = *height;
				}
				_ => {}
			}
		}
		events
	}

	/// Makes the window's GPU context current on the calling thread.
	pub fn make_current(&self) -> bool {
		match self.backend.as_ref() {
			Some(b) => b.make_current(),
			None => false,
		}
	}

	/// Swaps the front and back buffers, presenting the rendered frame.
	pub fn swap_buffers(&self) {
		if let Some(b) = self.backend.as_ref() {
			b.swap_buffers();
		}
	}

	/// Drains and returns absolute file system paths that were dropped onto
	/// this window since the last call. Returns an empty vector on platforms
	/// without a drag-and-drop backend or when no files were dropped.
	pub fn take_dropped_files(&mut self) -> Vec<String> {
		match self.backend.as_mut() {
			Some(b) => b.take_dropped_files(),
			None => Vec::new(),
		}
	}

	/// Resolves an OpenGL function pointer, caching the result. `name` must be
	/// NUL-terminated. Repeated calls for the same symbol hit a per-window
	/// in-memory cache instead of re-entering the OS loader.
	pub fn gl_get_proc(&self, name: &[u8]) -> *mut core::ffi::c_void {
		debug_assert_eq!(name.last(), Some(&0), "gl_get_proc name must be NUL-terminated");
		if let Ok(cache) = self.proc_cache.lock() {
			if let Some(hit) = cache.iter().find(|(k, _)| k.as_ref() == name) {
				return hit.1;
			}
		}
		let ptr = match self.backend.as_ref() {
			Some(b) => b.get_proc_address(name),
			None => core::ptr::null_mut(),
		};
		if let Ok(mut cache) = self.proc_cache.lock() {
			if !cache.iter().any(|(k, _)| k.as_ref() == name) {
				cache.push((name.to_vec().into_boxed_slice(), ptr));
			}
		}
		ptr
	}

	/// Resolves an OpenGL function pointer without consulting the cache, going
	/// straight to the OS loader on every call. Mostly useful for benchmarking
	/// or when intentionally re-resolving a symbol.
	pub fn gl_get_proc_uncached(&self, name: &[u8]) -> *mut core::ffi::c_void {
		match self.backend.as_ref() {
			Some(b) => b.get_proc_address(name),
			None => core::ptr::null_mut(),
		}
	}

	/// Returns true if a real GPU backend is attached.
	pub fn has_backend(&self) -> bool {
		self.backend.is_some()
	}

	/// Legacy software-presentation path (no GPU). Kept for backwards
	/// compatibility with existing callers; copies dimensions only.
	pub fn present_frame(&mut self, argb: &[u8], width: usize, height: usize) {
		if self.backend.is_none()
			&& self.vsync_capacity > 0
			&& argb.len() >= width.saturating_mul(height).saturating_mul(4)
		{
			self.width = width as u32;
			self.height = height as u32;
		}
	}
}

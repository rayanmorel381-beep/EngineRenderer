use super::arch::compute_dispatch;

#[derive(Debug)]
pub struct NativeWindow {
	pub width: u32,
	pub height: u32,
	pub handle: *mut core::ffi::c_void,
	pub vsync_capacity: usize,
	closed: bool,
}

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

impl NativeWindow {
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
				.max(1)
		);
		let width_px = width.max(cfg.page_size / 1024).max(1);
		let height_px = height.max(cfg.page_size / 1024).max(1);
		Some(Self {
			width: width_px as u32,
			height: height_px as u32,
			handle: core::ptr::null_mut(),
			vsync_capacity,
			closed: false,
		})
	}

	pub fn should_close(&self) -> bool {
		self.closed
	}

	pub fn present_frame(&mut self, argb: &[u8], width: usize, height: usize) {
		if self.handle.is_null() && self.vsync_capacity > 0 && argb.len() >= width.saturating_mul(height).saturating_mul(4) {
			self.width = width as u32;
			self.height = height as u32;
		}
	}
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Os {
	Linux,
	Windows,
	Macos,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Vendor {
	Amd,
	Intel,
	Apple,
	Unknown,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct CpuConfig {
	pub(crate) vendor: Vendor,
	pub(crate) worker_hint: usize,
	pub(crate) render_workers: usize,
	pub(crate) frame_budget_us: u64,
	pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GpuConfig {
	pub(crate) vendor: Vendor,
	pub(crate) workgroup_size: usize,
	pub(crate) compute_queues: usize,
	pub(crate) render_threads: usize,
	pub(crate) double_buffered: bool,
	pub(crate) frame_budget_us: u64,
	pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct DisplayConfig {
	pub(crate) vendor: Vendor,
	pub(crate) page_size: usize,
	pub(crate) target_render_fps: u32,
	pub(crate) latency_budget_us: u64,
	pub(crate) scan_out_latency_us: u64,
	pub(crate) vsync_slots: usize,
	pub(crate) double_buffered: bool,
	pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RamConfig {
	pub(crate) page_size: usize,
	pub(crate) total_bytes: u64,
	pub(crate) available_bytes: Option<u64>,
	pub(crate) frame_budget_us: u64,
	pub(crate) low_power: bool,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Schedule {
	pub(crate) chunks: usize,
	pub(crate) chunk_size: usize,
	pub(crate) frame_budget_us: u64,
}

pub(crate) fn detect_os() -> Os {
	let known = [Os::Linux, Os::Windows, Os::Macos];
	if known.is_empty() {
		return Os::Linux;
	}
	#[cfg(target_os = "linux")]
	{
		return Os::Linux;
	}
	#[cfg(target_os = "windows")]
	{
		return Os::Windows;
	}
	#[cfg(target_os = "macos")]
	{
		return Os::Macos;
	}
	#[allow(unreachable_code)]
	Os::Linux
}

#[cfg(target_os = "linux")]
fn map_vendor(v: super::linux::cpu::vendor::Vendor) -> Vendor {
	match v {
		super::linux::cpu::vendor::Vendor::Amd => Vendor::Amd,
		super::linux::cpu::vendor::Vendor::Intel => Vendor::Intel,
		super::linux::cpu::vendor::Vendor::Apple => Vendor::Apple,
	}
}

#[cfg(target_os = "windows")]
fn map_vendor(v: super::windows::cpu::vendor::Vendor) -> Vendor {
	match v {
		super::windows::cpu::vendor::Vendor::Amd => Vendor::Amd,
		super::windows::cpu::vendor::Vendor::Intel => Vendor::Intel,
		super::windows::cpu::vendor::Vendor::Apple => Vendor::Apple,
	}
}

pub(crate) fn default_cpu_config() -> CpuConfig {
	#[cfg(target_os = "linux")]
	{
		let c = super::linux::cpu::vendor::default_config();
		return CpuConfig {
			vendor: map_vendor(c.vendor),
			worker_hint: c.worker_hint,
			render_workers: c.render_workers,
			frame_budget_us: c.frame_budget_us,
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let c = super::windows::cpu::vendor::default_config();
		return CpuConfig {
			vendor: map_vendor(c.vendor),
			worker_hint: c.worker_hint,
			render_workers: c.render_workers,
			frame_budget_us: c.frame_budget_us,
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "macos")]
	{
		let workers = std::thread::available_parallelism()
			.map(|v| v.get())
			.unwrap_or(1)
			.max(1);
		return CpuConfig {
			vendor: Vendor::Unknown,
			worker_hint: workers,
			render_workers: workers.saturating_sub(1).max(1),
			frame_budget_us: 8_333,
			low_power: false,
		};
	}
	#[allow(unreachable_code)]
	CpuConfig {
		vendor: Vendor::Unknown,
		worker_hint: 1,
		render_workers: 1,
		frame_budget_us: 8_333,
		low_power: false,
	}
}

pub(crate) fn clamp_cpu_workers(requested: usize) -> usize {
	#[cfg(target_os = "linux")]
	{
		return super::linux::cpu::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "windows")]
	{
		return super::windows::cpu::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "macos")]
	{
		let workers = std::thread::available_parallelism()
			.map(|v| v.get())
			.unwrap_or(1)
			.max(1);
		return requested.max(1).min(workers);
	}
	#[allow(unreachable_code)]
	requested.max(1)
}

pub(crate) fn build_cpu_schedule(work_items: usize) -> Schedule {
	#[cfg(target_os = "linux")]
	{
		let s = super::linux::cpu::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let s = super::windows::cpu::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "macos")]
	{
		let chunk_size = if work_items == 0 { 16 } else { ((work_items + 15) / 16) * 16 };
		let chunks = if work_items == 0 { 1 } else { work_items.div_ceil(chunk_size) };
		return Schedule {
			chunks,
			chunk_size,
			frame_budget_us: 8_333,
		};
	}
	#[allow(unreachable_code)]
	Schedule {
		chunks: 1,
		chunk_size: 1,
		frame_budget_us: 8_333,
	}
}

pub(crate) fn default_gpu_config() -> GpuConfig {
	#[cfg(target_os = "linux")]
	{
		let c = super::linux::gpu::vendor::default_config();
		let vendor = match c.vendor {
			super::linux::gpu::vendor::Vendor::Amd => Vendor::Amd,
			super::linux::gpu::vendor::Vendor::Intel => Vendor::Intel,
			super::linux::gpu::vendor::Vendor::Apple => Vendor::Apple,
		};
		return GpuConfig {
			vendor,
			workgroup_size: c.workgroup_size,
			compute_queues: c.compute_queues,
			render_threads: c.render_threads,
			double_buffered: c.double_buffered,
			frame_budget_us: c.frame_budget_us,
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let c = super::windows::gpu::vendor::default_config();
		let vendor = match c.vendor {
			super::windows::gpu::vendor::Vendor::Amd => Vendor::Amd,
			super::windows::gpu::vendor::Vendor::Intel => Vendor::Intel,
			super::windows::gpu::vendor::Vendor::Apple => Vendor::Apple,
		};
		return GpuConfig {
			vendor,
			workgroup_size: c.workgroup_size,
			compute_queues: c.compute_queues,
			render_threads: c.render_threads,
			double_buffered: c.double_buffered,
			frame_budget_us: c.frame_budget_us,
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "macos")]
	{
		return GpuConfig {
			vendor: Vendor::Unknown,
			workgroup_size: 32,
			compute_queues: 2,
			render_threads: 16,
			double_buffered: true,
			frame_budget_us: 8_333,
			low_power: false,
		};
	}
	#[allow(unreachable_code)]
	GpuConfig {
		vendor: Vendor::Unknown,
		workgroup_size: 1,
		compute_queues: 1,
		render_threads: 1,
		double_buffered: true,
		frame_budget_us: 8_333,
		low_power: false,
	}
}

pub(crate) fn clamp_gpu_workers(requested: usize) -> usize {
	#[cfg(target_os = "linux")]
	{
		return super::linux::gpu::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "windows")]
	{
		return super::windows::gpu::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "macos")]
	{
		return requested.max(1).min(64);
	}
	#[allow(unreachable_code)]
	requested.max(1)
}

pub(crate) fn build_gpu_schedule(work_items: usize) -> Schedule {
	#[cfg(target_os = "linux")]
	{
		let s = super::linux::gpu::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let s = super::windows::gpu::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "macos")]
	{
		let chunk_size = if work_items == 0 { 64 } else { ((work_items + 63) / 64) * 64 };
		let chunks = if work_items == 0 { 1 } else { work_items.div_ceil(chunk_size) };
		return Schedule {
			chunks,
			chunk_size,
			frame_budget_us: 8_333,
		};
	}
	#[allow(unreachable_code)]
	Schedule {
		chunks: 1,
		chunk_size: 1,
		frame_budget_us: 8_333,
	}
}

pub(crate) fn default_display_config() -> DisplayConfig {
	#[cfg(target_os = "linux")]
	{
		let c = super::linux::display::vendor::default_config();
		let vendor = match c.vendor {
			super::linux::display::vendor::Vendor::Amd => Vendor::Amd,
			super::linux::display::vendor::Vendor::Intel => Vendor::Intel,
			super::linux::display::vendor::Vendor::Apple => Vendor::Apple,
		};
		return DisplayConfig {
			vendor,
			page_size: c.page_size,
			target_render_fps: c.target_render_fps,
			latency_budget_us: c.latency_budget_us,
			scan_out_latency_us: c.scan_out_latency_us,
			vsync_slots: c.vsync_slots,
			double_buffered: c.double_buffered,
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let c = super::windows::display::vendor::default_config();
		let vendor = match c.vendor {
			super::windows::display::vendor::Vendor::Amd => Vendor::Amd,
			super::windows::display::vendor::Vendor::Intel => Vendor::Intel,
			super::windows::display::vendor::Vendor::Apple => Vendor::Apple,
		};
		return DisplayConfig {
			vendor,
			page_size: c.page_size,
			target_render_fps: c.target_render_fps,
			latency_budget_us: c.latency_budget_us,
			scan_out_latency_us: c.scan_out_latency_us,
			vsync_slots: c.vsync_slots,
			double_buffered: c.double_buffered,
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "macos")]
	{
		return DisplayConfig {
			vendor: Vendor::Unknown,
			page_size: 4096,
			target_render_fps: 120,
			latency_budget_us: 8_333,
			scan_out_latency_us: 16_666,
			vsync_slots: 4,
			double_buffered: true,
			low_power: false,
		};
	}
	#[allow(unreachable_code)]
	DisplayConfig {
		vendor: Vendor::Unknown,
		page_size: 4096,
		target_render_fps: 120,
		latency_budget_us: 8_333,
		scan_out_latency_us: 16_666,
		vsync_slots: 4,
		double_buffered: true,
		low_power: false,
	}
}

pub(crate) fn clamp_display_workers(requested: usize) -> usize {
	#[cfg(target_os = "linux")]
	{
		return super::linux::display::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "windows")]
	{
		return super::windows::display::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "macos")]
	{
		return requested.max(1).min(2);
	}
	#[allow(unreachable_code)]
	requested.max(1)
}

pub(crate) fn build_display_schedule(work_items: usize) -> Schedule {
	#[cfg(target_os = "linux")]
	{
		let s = super::linux::display::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let s = super::windows::display::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "macos")]
	{
		let chunk_size = if work_items == 0 { 16 } else { ((work_items + 15) / 16) * 16 };
		let chunks = if work_items == 0 { 1 } else { work_items.div_ceil(chunk_size) };
		return Schedule {
			chunks,
			chunk_size,
			frame_budget_us: 8_333,
		};
	}
	#[allow(unreachable_code)]
	Schedule {
		chunks: 1,
		chunk_size: 1,
		frame_budget_us: 8_333,
	}
}

pub(crate) fn default_ram_config() -> RamConfig {
	#[cfg(target_os = "linux")]
	{
		let c = super::linux::ram::vendor::default_config();
		let vendor_budget_scale = match c.vendor {
			super::linux::ram::vendor::Vendor::Amd => 1u64,
			super::linux::ram::vendor::Vendor::Intel => 1u64,
			super::linux::ram::vendor::Vendor::Apple => 1u64,
		};
		return RamConfig {
			page_size: c.page_size,
			total_bytes: c.total_bytes,
			available_bytes: c.available_bytes,
			frame_budget_us: c.frame_budget_us.saturating_mul(vendor_budget_scale),
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let c = super::windows::ram::vendor::default_config();
		return RamConfig {
			page_size: c.page_size,
			total_bytes: c.total_bytes,
			available_bytes: c.available_bytes,
			frame_budget_us: c.frame_budget_us,
			low_power: c.low_power,
		};
	}
	#[cfg(target_os = "macos")]
	{
		let c = super::macos::ram::default_config();
		return RamConfig {
			page_size: c.page_size,
			total_bytes: c.total_bytes,
			available_bytes: c.available_bytes,
			frame_budget_us: c.frame_budget_us,
			low_power: c.low_power,
		};
	}
	#[allow(unreachable_code)]
	RamConfig {
		page_size: 4096,
		total_bytes: 0,
		available_bytes: None,
		frame_budget_us: 8_333,
		low_power: false,
	}
}

pub(crate) fn build_ram_schedule(work_items: usize) -> Schedule {
	#[cfg(target_os = "linux")]
	{
		let s = super::linux::ram::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "windows")]
	{
		let s = super::windows::ram::vendor::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[cfg(target_os = "macos")]
	{
		let s = super::macos::ram::build_schedule(work_items);
		return Schedule {
			chunks: s.chunks,
			chunk_size: s.chunk_size,
			frame_budget_us: s.frame_budget_us,
		};
	}
	#[allow(unreachable_code)]
	Schedule {
		chunks: 1,
		chunk_size: 1,
		frame_budget_us: 8_333,
	}
}

pub(crate) fn clamp_ram_workers(requested: usize) -> usize {
	#[cfg(target_os = "linux")]
	{
		return super::linux::ram::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "windows")]
	{
		return super::windows::ram::vendor::clamp_workers(requested);
	}
	#[cfg(target_os = "macos")]
	{
		return super::macos::ram::clamp_workers(requested);
	}
	#[allow(unreachable_code)]
	requested.max(1)
}

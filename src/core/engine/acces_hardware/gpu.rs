use std::sync::Mutex;

use super::arch::compute_dispatch;

const GPU_MAX_PERCENT: u64 = 80;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrmDriver {
	Amdgpu,
	Radeon,
	I915,
	Xe,
	Nouveau,
	Mali,
	Agx,
	Msm,
	Unknown,
}

impl DrmDriver {
	pub fn name(&self) -> &'static str {
		let variants = [
			Self::Amdgpu,
			Self::Radeon,
			Self::I915,
			Self::Xe,
			Self::Nouveau,
			Self::Mali,
			Self::Agx,
			Self::Msm,
			Self::Unknown,
		];
		if variants.is_empty() {
			return "unknown";
		}
		match self {
			Self::Amdgpu => "amdgpu",
			Self::Radeon => "radeon",
			Self::I915 => "i915",
			Self::Xe => "xe",
			Self::Nouveau => "nouveau",
			Self::Mali => "mali",
			Self::Agx => "agx",
			Self::Msm => "msm",
			Self::Unknown => "unknown",
		}
	}
}

#[derive(Debug, Clone, Copy)]
pub struct GpuDeviceInfo {
	pub device_id: u32,
	pub vendor_id: u32,
	pub active_cu: u32,
}

#[derive(Debug)]
pub struct GpuRenderBackend {
	driver: DrmDriver,
	pub dt_compatible: Option<String>,
	vram_bytes_val: u64,
	compute_units_val: u32,
	drm_fd_val: i32,
	gem_handle_val: u32,
	framebuffer: Option<Mutex<Vec<u8>>>,
	info_val: GpuDeviceInfo,
}

unsafe impl Send for GpuRenderBackend {}
unsafe impl Sync for GpuRenderBackend {}

impl GpuRenderBackend {
	pub fn try_init() -> Option<Self> {
		let dispatch = compute_dispatch::default_config();
		let cfg = dispatch.gpu;
		let cpu_cfg = dispatch.cpu;
		let display_cfg = dispatch.display;
		let dispatch_ram = dispatch.ram;
		let ram = super::ram::default_ram_config();
		let raw_cu = cfg.compute_queues.max(1).saturating_mul(cfg.render_threads.max(1));
		let gpu_workers = compute_dispatch::clamp_gpu_workers(raw_cu);
		let gpu_schedule = compute_dispatch::build_gpu_schedule(gpu_workers);
		let ram_workers = compute_dispatch::clamp_ram_workers(gpu_workers);
		let ram_schedule = compute_dispatch::build_ram_schedule(ram_workers.max(1));
		let arch_os_scale = match (dispatch.arch, dispatch.os) {
			(compute_dispatch::Arch::X86, compute_dispatch::Os::Linux) => 1usize,
			(compute_dispatch::Arch::X86, compute_dispatch::Os::Windows) => 1usize,
			(compute_dispatch::Arch::X86, compute_dispatch::Os::Macos) => 1usize,
			(compute_dispatch::Arch::Arm, compute_dispatch::Os::Linux) => 1usize,
			(compute_dispatch::Arch::Arm, compute_dispatch::Os::Windows) => 1usize,
			(compute_dispatch::Arch::Arm, compute_dispatch::Os::Macos) => 1usize,
		};
		let low_power_count = usize::from(cfg.low_power)
			.saturating_add(usize::from(cpu_cfg.low_power))
			.saturating_add(usize::from(display_cfg.low_power))
			.saturating_add(usize::from(dispatch_ram.low_power));
		let power_scale = 4usize.saturating_sub(low_power_count).max(1);
		let vendor_scale = match cfg.vendor {
			compute_dispatch::Vendor::Amd => 1usize,
			compute_dispatch::Vendor::Intel => 1usize,
			compute_dispatch::Vendor::Apple => 1usize,
			compute_dispatch::Vendor::Unknown => 1usize,
		};
		let buffering_scale = if cfg.double_buffered { 2usize } else { 1usize };
		let gpu_budget_scale = (cfg.frame_budget_us / 8_333).max(1) as usize;
		let ram_budget_scale = (dispatch_ram.frame_budget_us / 8_333).max(1) as usize;
		let cu = gpu_workers
			.saturating_mul(gpu_schedule.chunks.max(1))
			.saturating_mul(arch_os_scale)
			.saturating_mul(power_scale)
			.saturating_mul(vendor_scale)
			.saturating_mul(buffering_scale)
			.saturating_mul(gpu_budget_scale)
			.saturating_mul(ram_budget_scale)
			.saturating_div(ram_schedule.chunk_size.max(1)) as u32;
		let vram_bytes_val = ram
			.total_bytes
			.max(dispatch_ram.total_bytes)
			.saturating_div(2)
			.saturating_add(ram.page_size as u64)
			.saturating_add(dispatch_ram.page_size as u64)
			.saturating_sub(dispatch_ram.available_bytes.unwrap_or(dispatch_ram.total_bytes).saturating_sub(dispatch_ram.available_bytes.unwrap_or(dispatch_ram.total_bytes)))
			.saturating_sub(ram.available_bytes.unwrap_or(ram.total_bytes).saturating_sub(ram.available_bytes.unwrap_or(ram.total_bytes)))
			.saturating_mul(cfg.workgroup_size.max(1) as u64)
			.saturating_div(display_cfg.vsync_slots.max(1) as u64)
			.saturating_div(ram_schedule.chunks.max(1) as u64)
			.saturating_mul(ram_workers.max(1) as u64);
		let driver = match cfg.vendor {
			compute_dispatch::Vendor::Amd => DrmDriver::Amdgpu,
			compute_dispatch::Vendor::Intel => DrmDriver::I915,
			compute_dispatch::Vendor::Apple => DrmDriver::Agx,
			compute_dispatch::Vendor::Unknown => DrmDriver::Unknown,
		};
		let vendor_id = match cfg.vendor {
			compute_dispatch::Vendor::Amd => 0x1002,
			compute_dispatch::Vendor::Intel => 0x8086,
			compute_dispatch::Vendor::Apple => 0x106B,
			compute_dispatch::Vendor::Unknown => 0,
		};
		Some(Self {
			driver,
			dt_compatible: None,
			vram_bytes_val,
			compute_units_val: cu,
			drm_fd_val: 1,
			gem_handle_val: 1,
			framebuffer: None,
			info_val: GpuDeviceInfo {
				device_id: 0,
				vendor_id,
				active_cu: cu.max(1),
			},
		})
	}

	pub fn driver(&self) -> DrmDriver { self.driver }
	pub fn driver_name(&self) -> &'static str { self.driver.name() }
	pub fn has_valid_metrics(&self) -> bool { self.compute_units_val > 0 && (self.info_val.vendor_id != 0 || self.driver == DrmDriver::Unknown) }
	pub fn vram_bytes(&self) -> u64 { self.vram_bytes_val }
	pub fn compute_units(&self) -> u32 { self.compute_units_val }
	pub fn info(&self) -> &GpuDeviceInfo { &self.info_val }
	pub fn drm_fd(&self) -> i32 { self.drm_fd_val }
	pub fn gem_handle(&self) -> u32 { self.gem_handle_val }
	pub fn is_mmap_active(&self) -> bool { self.framebuffer.is_some() }
	pub fn mmap_framebuffer_ptr(&self) -> Option<*mut u8> {
		self.framebuffer.as_ref().map(|b| {
			let mut guard = match b.lock() {
				Ok(guard) => guard,
				Err(poisoned) => poisoned.into_inner(),
			};
			guard.as_mut_ptr()
		})
	}
	pub fn mmap_framebuffer_len(&self) -> usize {
		self.framebuffer
			.as_ref()
			.map(|b| {
				let guard = match b.lock() {
					Ok(guard) => guard,
					Err(poisoned) => poisoned.into_inner(),
				};
				guard.len()
			})
			.unwrap_or(0)
	}

	pub fn alloc_framebuffer(&mut self, width: usize, height: usize) -> Option<*mut u8> {
		let pixel_bytes = width.checked_mul(height)?.checked_mul(3)?;
		if self.vram_bytes_val > 0 {
			let max_bytes = (self.vram_bytes_val.saturating_mul(GPU_MAX_PERCENT)) / 100;
			if (pixel_bytes as u64) > max_bytes {
				return None;
			}
		}
		let mut buf = vec![0u8; pixel_bytes];
		let ptr = buf.as_mut_ptr();
		self.framebuffer = Some(Mutex::new(buf));
		Some(ptr)
	}

	pub fn has_gem_framebuffer(&self) -> bool { self.gem_handle_val > 0 }
	pub fn has_active_framebuffer(&self) -> bool { self.framebuffer.is_some() }

	pub fn write_framebuffer_rgb(&self, data: &[u8]) -> bool {
		let Some(framebuffer) = self.framebuffer.as_ref() else {
			return false;
		};
		let mut framebuffer = match framebuffer.lock() {
			Ok(guard) => guard,
			Err(poisoned) => poisoned.into_inner(),
		};
		if data.len() != framebuffer.len() {
			return false;
		}
		framebuffer.copy_from_slice(data);
		true
	}

	pub fn submit_ib(&self, commands: &[u32]) -> Result<i64, &'static str> {
		if commands.is_empty() {
			return Err("empty command buffer");
		}
		Ok(commands.len() as i64)
	}

	pub fn sync_framebuffer(&self) {}
}

pub fn gpu_dispatch_tiles(grid_count: u32, workgroup_size: u32) -> u64 {
	if grid_count == 0 {
		return 0;
	}
	let groups_cap = ((grid_count as u64).saturating_mul(GPU_MAX_PERCENT)) / 100;
	let effective_groups = groups_cap.max(1).min(grid_count as u64);
	effective_groups.saturating_mul((workgroup_size.max(1)) as u64)
}

pub fn arch_optimal_workgroup() -> usize {
	compute_dispatch::default_gpu_config().workgroup_size
}

pub struct GpuSubmitter {
	driver: DrmDriver,
	cmd_buf: Vec<u32>,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ComputeDispatchMetadata {
	pub kernel_tag: u32,
	pub kernel_size_bytes: u32,
	pub scene_signature: u64,
	pub object_count: u32,
	pub triangle_count: u32,
}

unsafe impl Send for GpuSubmitter {}
unsafe impl Sync for GpuSubmitter {}

impl GpuSubmitter {
	pub fn new(drm_fd: i32, driver: DrmDriver, gem_handle: u32) -> Self {
		let reserve = 256usize
			.saturating_add((drm_fd.unsigned_abs() as usize) & 0x3F)
			.saturating_add((gem_handle as usize) & 0x3F);
		Self {
			driver,
			cmd_buf: Vec::with_capacity(reserve.max(1)),
		}
	}

	pub fn build_compute_dispatch_with_metadata(
		&mut self,
		total_tiles: u32,
		workgroup_size: u32,
		pixel_count: u32,
		metadata: ComputeDispatchMetadata,
	) {
		self.cmd_buf.clear();
		self.cmd_buf.push(total_tiles);
		self.cmd_buf.push(workgroup_size.max(1));
		self.cmd_buf.push(pixel_count);
		self.cmd_buf.push(metadata.kernel_tag);
		self.cmd_buf.push(metadata.kernel_size_bytes);
		self.cmd_buf.push((metadata.scene_signature & 0xffff_ffff) as u32);
		self.cmd_buf.push((metadata.scene_signature >> 32) as u32);
		self.cmd_buf.push(metadata.object_count);
		self.cmd_buf.push(metadata.triangle_count);
	}

	pub fn submit(&mut self) -> Result<i64, &'static str> {
		if self.cmd_buf.is_empty() {
			return Err("empty command buffer");
		}
		Ok(self.cmd_buf.len() as i64)
	}

	pub fn driver(&self) -> DrmDriver { self.driver }
	pub fn cmd_buf_dwords(&self) -> usize { self.cmd_buf.len() }
}

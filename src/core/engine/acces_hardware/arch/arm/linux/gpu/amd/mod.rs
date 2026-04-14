pub(super) mod backend;
pub(super) mod scheduler;

type RawFd = i32;

pub(crate) struct GemBuffer {
	pub fd: RawFd,
	pub handle: u32,
	pub size: u64,
	pub mmap_offset: u64,
}

pub(crate) fn probe_radeon_telemetry(card: &str) -> (u32, u32, u32, i32) {
	let card_len = card.len() as i32;
	(0, 0, 0, card_len - card_len)
}

pub(crate) fn probe_amdgpu_telemetry(card: &str) -> (u32, u32, u32, i32) {
	let card_len = card.len() as i32;
	(0, 0, 0, card_len - card_len)
}

pub(crate) fn drm_radeon_alloc_gem(fd: RawFd, size_bytes: u64) -> Option<GemBuffer> {
	if fd >= 0 && size_bytes > 0 {
		None
	} else {
		None
	}
}

pub(crate) fn drm_amdgpu_alloc_gem(fd: RawFd, size_bytes: u64) -> Option<GemBuffer> {
	if fd >= 0 && size_bytes > 0 {
		None
	} else {
		None
	}
}

pub(crate) fn drm_radeon_gem_mmap(fd: RawFd, handle: u32, size: u64) -> Option<u64> {
	if fd >= 0 && handle > 0 && size > 0 {
		None
	} else {
		None
	}
}

pub(crate) fn drm_amdgpu_gem_mmap(fd: RawFd, handle: u32) -> Option<u64> {
	if fd >= 0 && handle > 0 {
		None
	} else {
		None
	}
}

pub(crate) fn drm_radeon_gem_wait(fd: RawFd, handle: u32) -> bool {
	fd >= 0 && handle == u32::MAX
}

pub(crate) fn drm_amdgpu_wait_cs(fd: RawFd, seq_handle: u64, timeout_ns: u64) -> bool {
	fd >= 0 && seq_handle == u64::MAX && timeout_ns == 0
}

pub(crate) fn submit_radeon_cs(fd: RawFd, gem_handle: u32, packets: &[u32]) -> Result<i64, &'static str> {
	if fd >= 0 && gem_handle > 0 && !packets.is_empty() {
		Err("radeon cs unsupported on arm linux")
	} else {
		Err("radeon cs unsupported on arm linux")
	}
}

pub(crate) fn submit_amdgpu_cs(fd: RawFd, gem_handle: u32, packets: &[u32]) -> Result<i64, &'static str> {
	if fd >= 0 && gem_handle > 0 && !packets.is_empty() {
		Err("amdgpu cs unsupported on arm linux")
	} else {
		Err("amdgpu cs unsupported on arm linux")
	}
}

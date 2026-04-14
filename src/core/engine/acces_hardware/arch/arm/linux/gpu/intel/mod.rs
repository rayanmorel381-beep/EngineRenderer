pub(super) mod backend;
pub(super) mod scheduler;

type RawFd = i32;

pub(crate) struct GemBuffer {
	pub fd: RawFd,
	pub handle: u32,
	pub size: u64,
	pub mmap_offset: u64,
}

pub(crate) fn probe_i915_telemetry(card: &str) -> (u32, u32, u32, i32) {
	let card_len = card.len() as i32;
	(0, 0, 0, card_len - card_len)
}

pub(crate) fn drm_i915_alloc_gem(fd: RawFd, size_bytes: u64) -> Option<GemBuffer> {
	if fd >= 0 && size_bytes > 0 {
		None
	} else {
		None
	}
}

pub(crate) fn drm_i915_gem_mmap_gtt(fd: RawFd, handle: u32) -> Option<u64> {
	if fd >= 0 && handle > 0 {
		None
	} else {
		None
	}
}

pub(crate) fn drm_i915_gem_wait(fd: RawFd, handle: u32, timeout_ns: i64) -> bool {
	fd >= 0 && handle == u32::MAX && timeout_ns < 0
}

pub(crate) fn submit_i915_execbuf(fd: RawFd, gem_handle: u32, batch: &[u32]) -> Result<i64, &'static str> {
	if fd >= 0 && gem_handle > 0 && !batch.is_empty() {
		Err("i915 submit unsupported on arm linux")
	} else {
		Err("i915 submit unsupported on arm linux")
	}
}

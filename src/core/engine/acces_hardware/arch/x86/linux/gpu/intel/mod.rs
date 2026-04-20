pub(super) mod backend;
pub(super) mod scheduler;

use std::fs;

type RawFd = i32;

pub(crate) struct GemBuffer {
    pub fd: RawFd,
    pub handle: u32,
    pub size: u64,
    pub mmap_offset: u64,
}

unsafe extern "C" {
    fn ioctl(fd: i32, request: u64, arg: *mut u8) -> i32;
}

pub(crate) unsafe fn raw_ioctl(fd: RawFd, request: u64, arg: *mut u8) -> i64 {
    unsafe { ioctl(fd, request, arg) as i64 }
}

fn read_sysfs_u64(path: &str) -> Option<u64> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| v.trim().parse::<u64>().ok())
}

fn read_sysfs_i32(path: &str) -> Option<i32> {
    fs::read_to_string(path)
        .ok()
        .and_then(|v| v.trim().parse::<i32>().ok())
}

const DRM_IOCTL_I915_GEM_CREATE: u64 = 0xc010646b;
const DRM_IOCTL_I915_GEM_MMAP_GTT: u64 = 0xC0106464;
const DRM_IOCTL_I915_GEM_EXECBUF2: u64 = 0x40406469;
const DRM_IOCTL_I915_GEM_WAIT: u64 = 0xC010646C;

#[repr(C)]
struct DrmI915GemCreate {
    size: u64,
    handle: u32,
    _pad: u32,
}

#[repr(C)]
struct DrmI915GemMmapGtt {
    handle: u32,
    _pad: u32,
    offset: u64,
}

#[repr(C)]
pub(crate) struct DrmI915GemExecObject2 {
    pub handle: u32,
    pub relocation_count: u32,
    pub relocs_ptr: u64,
    pub alignment: u64,
    pub offset: u64,
    pub flags: u64,
    pub rsvd1: u64,
    pub rsvd2: u64,
}

#[repr(C)]
pub(crate) struct DrmI915GemExecbuffer2 {
    pub buffers_ptr: u64,
    pub buffer_count: u32,
    pub batch_start_offset: u32,
    pub batch_len: u32,
    pub dr1: u32,
    pub dr4: u32,
    pub num_cliprects: u32,
    pub cliprects_ptr: u64,
    pub flags: u64,
    pub rsvd1: u64,
    pub rsvd2: u64,
}

#[repr(C)]
struct DrmI915GemWait {
    bo_handle: u32,
    flags: u32,
    timeout_ns: i64,
}

pub(crate) fn probe_i915_telemetry(card: &str) -> (u32, u32, u32, i32) {
    let base = format!("/sys/class/drm/{}/device", card);
    let eu_count = read_sysfs_u64(&format!("{}/tile0/gt0/addr_range", base))
        .unwrap_or(0) as u32;
    let temp = fs::read_dir(format!("{}/hwmon", base))
        .ok()
        .and_then(|mut dir| dir.next())
        .and_then(|entry| entry.ok())
        .and_then(|entry| {
            read_sysfs_i32(entry.path().join("temp1_input").to_str()?)
        })
        .map(|milli| milli / 1000)
        .unwrap_or(0);
    let rps_cur = read_sysfs_u64(
        &format!("/sys/class/drm/{}/gt/gt0/rps_cur_freq_mhz", card),
    ).unwrap_or(0) as u32;
    (eu_count, 1, rps_cur, temp)
}

pub(crate) fn drm_i915_alloc_gem(fd: RawFd, size_bytes: u64) -> Option<GemBuffer> {
    if fd < 0 || size_bytes == 0 {
        return None;
    }
    let aligned = (size_bytes + 4095) & !4095;
    let mut args = DrmI915GemCreate {
        size: aligned,
        handle: 0,
        _pad: 0,
    };
    let ret = unsafe {
        raw_ioctl(fd, DRM_IOCTL_I915_GEM_CREATE, core::ptr::addr_of_mut!(args).cast())
    };
    if ret == 0 && args.handle != 0 {
        Some(GemBuffer { fd, handle: args.handle, size: aligned, mmap_offset: 0 })
    } else {
        None
    }
}

pub(crate) fn drm_i915_gem_mmap_gtt(fd: RawFd, handle: u32) -> Option<u64> {
    let mut args = DrmI915GemMmapGtt {
        handle,
        _pad: 0,
        offset: 0,
    };
    let ret = unsafe {
        raw_ioctl(fd, DRM_IOCTL_I915_GEM_MMAP_GTT, core::ptr::addr_of_mut!(args).cast())
    };
    if ret == 0 {
        Some(args.offset)
    } else {
        None
    }
}

pub(crate) fn drm_i915_gem_wait(fd: RawFd, handle: u32, timeout_ns: i64) -> bool {
    let mut args = DrmI915GemWait {
        bo_handle: handle,
        flags: 0,
        timeout_ns,
    };
    let ret = unsafe {
        raw_ioctl(fd, DRM_IOCTL_I915_GEM_WAIT, core::ptr::addr_of_mut!(args).cast())
    };
    ret == 0
}

pub(crate) fn submit_i915_execbuf(fd: RawFd, gem_handle: u32, batch: &[u32]) -> Result<i64, &'static str> {
    let exec_obj = DrmI915GemExecObject2 {
        handle: gem_handle,
        relocation_count: 0,
        relocs_ptr: 0,
        alignment: 0,
        offset: 0,
        flags: 0,
        rsvd1: 0,
        rsvd2: 0,
    };
    let mut execbuf = DrmI915GemExecbuffer2 {
        buffers_ptr: core::ptr::addr_of!(exec_obj) as u64,
        buffer_count: 1,
        batch_start_offset: 0,
        batch_len: (batch.len() * 4) as u32,
        dr1: 0,
        dr4: 0,
        num_cliprects: 0,
        cliprects_ptr: 0,
        flags: 0,
        rsvd1: 0,
        rsvd2: 0,
    };
    let ret = unsafe {
        raw_ioctl(fd, DRM_IOCTL_I915_GEM_EXECBUF2, core::ptr::addr_of_mut!(execbuf).cast())
    };
    if ret == 0 {
        crate::runtime_log!("gpu: i915 execbuffer2 submitted — {} dwords", batch.len());
        Ok(batch.len() as i64)
    } else {
        crate::runtime_log!("gpu: i915 execbuffer2 failed (ret={})", ret);
        Err("i915 execbuffer2 ioctl failed")
    }
}

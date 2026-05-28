//! Modern `amdgpu` kernel driver bindings (GCN 1.2+ and all RDNA generations).
//! Uses the `DRM_IOCTL_AMDGPU_*` ioctl family.

use std::fs;

use super::drm_ffi::{GemBuffer, RawFd, raw_ioctl, read_sysfs_i32, read_sysfs_u64};

const DRM_IOCTL_AMDGPU_GEM_CREATE: u64 = 0xc0206440;
const DRM_IOCTL_AMDGPU_GEM_MMAP: u64 = 0xC0106445;
const DRM_IOCTL_AMDGPU_CS: u64 = 0xC0206444;
const DRM_IOCTL_AMDGPU_WAIT_CS: u64 = 0xC0206449;

#[repr(C)]
struct DrmAmdgpuGemCreate {
    bo_size: u64,
    alignment: u64,
    domains: u64,
    domain_flags: u64,
}

#[repr(C)]
struct DrmAmdgpuGemMmap {
    in_handle: u32,
    pad: u32,
    out_addr_ptr: u64,
}

#[repr(C)]
pub(crate) struct DrmAmdgpuCsChunk {
    pub chunk_id: u32,
    pub length_dw: u32,
    pub chunk_data: u64,
}

#[repr(C)]
pub(crate) struct DrmAmdgpuCsIn {
    pub ctx_id: u32,
    pub bo_list_handle: u32,
    pub num_chunks: u32,
    pub flags: u32,
    pub chunks: u64,
}

#[repr(C)]
struct DrmAmdgpuWaitCs {
    handle: u64,
    timeout: u64,
    ip_type: u32,
    ip_instance: u32,
    ring: u32,
    ctx_id: u32,
    status: u64,
}

pub(crate) fn probe_amdgpu_telemetry(card: &str) -> (u32, u32, u32, i32) {
    let base = format!("/sys/class/drm/{}/device", card);
    let active_cu = read_sysfs_u64(&format!("{}/num_cu", base)).unwrap_or(0) as u32;
    let shader_engines =
        read_sysfs_u64(&format!("{}/num_shader_engines", base)).unwrap_or(0) as u32;
    let sclk = read_sysfs_u64(&format!("{}/pp_dpm_sclk", base))
        .and_then(|_| fs::read_to_string(format!("{}/pp_dpm_sclk", base)).ok())
        .and_then(|content| {
            content.lines().filter(|l| l.contains('*')).find_map(|l| {
                l.split_whitespace()
                    .find_map(|w| w.trim_end_matches("Mhz").parse::<u64>().ok())
            })
        })
        .unwrap_or(0) as u32;
    let temp = fs::read_dir(format!("{}/hwmon", base))
        .ok()
        .and_then(|mut dir| dir.next())
        .and_then(|entry| entry.ok())
        .and_then(|entry| {
            let hwmon = entry.path();
            read_sysfs_i32(hwmon.join("temp1_input").to_str()?)
        })
        .map(|milli| milli / 1000)
        .unwrap_or(0);
    (active_cu, shader_engines, sclk, temp)
}

pub(crate) fn drm_amdgpu_alloc_gem(fd: RawFd, size_bytes: u64) -> Option<GemBuffer> {
    if fd < 0 || size_bytes == 0 {
        return None;
    }
    let aligned = (size_bytes + 4095) & !4095;
    let mut args = DrmAmdgpuGemCreate {
        bo_size: aligned,
        alignment: 4096,
        domains: 0x2,
        domain_flags: 0,
    };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_AMDGPU_GEM_CREATE,
            core::ptr::addr_of_mut!(args).cast(),
        )
    };
    let handle = args.bo_size as u32;
    if ret == 0 && handle != 0 {
        Some(GemBuffer {
            fd,
            handle,
            size: aligned,
            mmap_offset: 0,
        })
    } else {
        None
    }
}

pub(crate) fn drm_amdgpu_gem_mmap(fd: RawFd, handle: u32) -> Option<u64> {
    let mut args = DrmAmdgpuGemMmap {
        in_handle: handle,
        pad: 0,
        out_addr_ptr: 0,
    };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_AMDGPU_GEM_MMAP,
            core::ptr::addr_of_mut!(args).cast(),
        )
    };
    if ret == 0 && args.out_addr_ptr != 0 {
        Some(args.out_addr_ptr)
    } else {
        None
    }
}

pub(crate) fn drm_amdgpu_wait_cs(fd: RawFd, seq_handle: u64, timeout_ns: u64) -> bool {
    let mut args = DrmAmdgpuWaitCs {
        handle: seq_handle,
        timeout: timeout_ns,
        ip_type: 0,
        ip_instance: 0,
        ring: 0,
        ctx_id: 0,
        status: 0,
    };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_AMDGPU_WAIT_CS,
            core::ptr::addr_of_mut!(args).cast(),
        )
    };
    ret == 0 && args.status == 0
}

pub(crate) fn submit_amdgpu_cs(
    fd: RawFd,
    gem_handle: u32,
    packets: &[u32],
) -> Result<i64, &'static str> {
    let ib_chunk = DrmAmdgpuCsChunk {
        chunk_id: 0x01,
        length_dw: packets.len() as u32,
        chunk_data: packets.as_ptr() as u64,
    };
    let chunk_ptrs: [u64; 1] = [core::ptr::addr_of!(ib_chunk) as u64];
    let mut cs_in = DrmAmdgpuCsIn {
        ctx_id: 0,
        bo_list_handle: 0,
        num_chunks: 1,
        flags: 0,
        chunks: chunk_ptrs.as_ptr() as u64,
    };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_AMDGPU_CS,
            core::ptr::addr_of_mut!(cs_in).cast(),
        )
    };
    if ret == 0 {
        crate::runtime_log!(
            "gpu: amdgpu CS submitted — {} PM4 dwords, gem_handle={}",
            packets.len(),
            gem_handle,
        );
        Ok(packets.len() as i64)
    } else {
        crate::runtime_log!("gpu: amdgpu CS ioctl failed (ret={})", ret);
        Err("amdgpu cs ioctl failed")
    }
}

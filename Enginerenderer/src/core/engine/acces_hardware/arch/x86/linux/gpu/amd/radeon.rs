//! Legacy `radeon` kernel driver bindings (pre-GCN through Southern/Sea/Volcanic
//! Islands ASICs). Uses the `DRM_IOCTL_RADEON_*` ioctl family.

use std::fs;

use super::drm_ffi::{GemBuffer, RawFd, raw_ioctl, read_sysfs_i32};

const DRM_IOCTL_RADEON_GEM_CREATE: u64 = 0xC01C645D;
const DRM_IOCTL_RADEON_GEM_MMAP: u64 = 0xC020645E;
const DRM_IOCTL_RADEON_GEM_SET_DOMAIN: u64 = 0x4010645F;
const DRM_IOCTL_RADEON_GEM_WAIT_IDLE: u64 = 0x40086464;
const DRM_IOCTL_RADEON_CS: u64 = 0xC0206466;

#[repr(C)]
struct DrmRadeonGemCreate {
    size: u64,
    alignment: u64,
    handle: u32,
    initial_domain: u32,
    flags: u32,
}

#[repr(C)]
struct DrmRadeonGemMmap {
    handle: u32,
    pad: u32,
    offset: u64,
    size: u64,
    addr_ptr: u64,
}

#[repr(C)]
struct DrmRadeonGemWait {
    handle: u32,
    pad: u32,
}

#[repr(C)]
struct DrmRadeonGemSetDomain {
    handle: u32,
    read_domains: u32,
    write_domain: u32,
    pad: u32,
}

#[repr(C)]
pub(crate) struct DrmRadeonCsChunk {
    pub chunk_id: u32,
    pub length_dw: u32,
    pub chunk_data: u64,
}

#[repr(C)]
pub(crate) struct DrmRadeonCs {
    pub num_chunks: u32,
    pub cs_id: u32,
    pub chunks: u64,
    pub gart_limit: u64,
    pub vram_limit: u64,
}

pub(crate) fn probe_radeon_telemetry(card: &str) -> (u32, u32, u32, i32) {
    let base = format!("/sys/class/drm/{}/device", card);

    let uevent = fs::read_to_string(format!("{}/uevent", base)).unwrap_or_default();
    let mut device_id: u32 = 0;
    for line in uevent.lines() {
        if let Some(val) = line.strip_prefix("PCI_ID=")
            && let Some(dev) = val.split(':').nth(1)
        {
            device_id = u32::from_str_radix(dev.trim(), 16).unwrap_or(0);
        }
    }

    let active_cu = match device_id {
        0x6819 | 0x6818 => 20,
        0x6810 | 0x6811 => 28,
        0x6800 | 0x6801 => 32,
        0x6820 | 0x6821 => 10,
        0x6835 | 0x6837 => 14,
        0x6660 | 0x6663 => 8,
        0x6640 | 0x6649 => 5,
        0x6610 | 0x6611 => 6,
        0x6600 | 0x6601 => 8,
        0x6790 | 0x6798 | 0x6799 => 32,
        0x6780 | 0x6788 | 0x6789 => 44,
        _ => 16,
    };

    let temp = fs::read_dir(format!("{}/hwmon", base))
        .ok()
        .and_then(|mut dir| dir.next())
        .and_then(|entry| entry.ok())
        .and_then(|entry| read_sysfs_i32(entry.path().join("temp1_input").to_str()?))
        .map(|milli| milli / 1000)
        .unwrap_or(0);

    (active_cu, 1, 0, temp)
}

pub(crate) fn drm_radeon_alloc_gem(fd: RawFd, size_bytes: u64) -> Option<GemBuffer> {
    if fd < 0 || size_bytes == 0 {
        return None;
    }
    let aligned = (size_bytes + 4095) & !4095;
    let mut args = DrmRadeonGemCreate {
        size: aligned,
        alignment: 4096,
        handle: 0,
        initial_domain: 0x2,
        flags: 0,
    };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_RADEON_GEM_CREATE,
            core::ptr::addr_of_mut!(args).cast(),
        )
    };
    if ret == 0 && args.handle != 0 {
        crate::runtime_log!(
            "gpu: radeon GEM created — handle={} size={}KB",
            args.handle,
            aligned / 1024
        );
        Some(GemBuffer {
            fd,
            handle: args.handle,
            size: aligned,
            mmap_offset: 0,
        })
    } else {
        crate::runtime_log!("gpu: radeon GEM create failed (ret={})", ret);
        None
    }
}

pub(crate) fn drm_radeon_gem_mmap(fd: RawFd, handle: u32, size: u64) -> Option<u64> {
    let mut args = DrmRadeonGemMmap {
        handle,
        pad: 0,
        offset: 0,
        size,
        addr_ptr: 0,
    };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_RADEON_GEM_MMAP,
            core::ptr::addr_of_mut!(args).cast(),
        )
    };
    if ret == 0 && args.addr_ptr != 0 {
        Some(args.addr_ptr)
    } else {
        crate::runtime_log!("gpu: radeon GEM mmap ioctl failed (ret={})", ret);
        None
    }
}

pub(crate) fn drm_radeon_gem_wait(fd: RawFd, handle: u32) -> bool {
    let mut args = DrmRadeonGemWait { handle, pad: 0 };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_RADEON_GEM_WAIT_IDLE,
            core::ptr::addr_of_mut!(args).cast(),
        )
    };
    ret == 0
}

pub(crate) fn drm_radeon_set_domain(
    fd: RawFd,
    handle: u32,
    read_domains: u32,
    write_domain: u32,
) -> bool {
    let mut args = DrmRadeonGemSetDomain {
        handle,
        read_domains,
        write_domain,
        pad: 0,
    };
    let ret = unsafe {
        raw_ioctl(
            fd,
            DRM_IOCTL_RADEON_GEM_SET_DOMAIN,
            core::ptr::addr_of_mut!(args).cast(),
        )
    };
    ret == 0
}

pub(crate) fn submit_radeon_cs(
    fd: RawFd,
    gem_handle: u32,
    packets: &[u32],
) -> Result<i64, &'static str> {
    drm_radeon_set_domain(fd, gem_handle, 0x2, 0x2);
    let relocs_chunk = DrmRadeonCsChunk {
        chunk_id: 0x01,
        length_dw: 0,
        chunk_data: 0,
    };
    let ib_chunk = DrmRadeonCsChunk {
        chunk_id: 0x02,
        length_dw: packets.len() as u32,
        chunk_data: packets.as_ptr() as u64,
    };
    let chunk_ptrs: [u64; 2] = [
        core::ptr::addr_of!(relocs_chunk) as u64,
        core::ptr::addr_of!(ib_chunk) as u64,
    ];
    let mut cs = DrmRadeonCs {
        num_chunks: 2,
        cs_id: 0,
        chunks: chunk_ptrs.as_ptr() as u64,
        gart_limit: 0,
        vram_limit: 0,
    };
    let ret = unsafe { raw_ioctl(fd, DRM_IOCTL_RADEON_CS, core::ptr::addr_of_mut!(cs).cast()) };
    if ret == 0 {
        crate::runtime_log!(
            "gpu: radeon CS submitted — {} dwords, cs_id={}, gem_handle={}",
            packets.len(),
            cs.cs_id,
            gem_handle,
        );
        Ok(cs.cs_id as i64)
    } else {
        crate::runtime_log!("gpu: radeon CS ioctl failed (ret={}, errno={})", ret, -ret);
        Err("radeon cs ioctl failed")
    }
}

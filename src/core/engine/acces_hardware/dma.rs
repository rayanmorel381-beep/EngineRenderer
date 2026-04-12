//! DMA-coherent framebuffer allocation for zero-copy GPU ↔ CPU transfers.

use hardware::sys;

/// Allocates a DMA-coherent buffer for the framebuffer.
///
/// The buffer is page-aligned and suitable for zero-copy transfers
/// between CPU and GPU.
pub fn alloc_dma_framebuffer(width: usize, height: usize) -> Option<DmaFramebuffer> {
    let pixel_bytes = width * height * 3 * 8; // 3 channels × f64
    let align = 4096; // page-aligned
    let buf = sys::dma::buffer::DmaBuffer::new(pixel_bytes, align)?;
    Some(DmaFramebuffer {
        width,
        height,
        buffer: buf,
    })
}

/// DMA-backed framebuffer for zero-copy GPU ↔ CPU transfers.
pub struct DmaFramebuffer {
    pub width: usize,
    pub height: usize,
    buffer: sys::DmaBuffer,
}

/// SAFETY: DmaFramebuffer is only accessed from the main thread.
/// It is stored in TileScheduler which is shared via `&self` during
/// dispatch, but worker threads never touch the DMA buffer — they
/// only call `tile_at()` which reads image dimensions.
unsafe impl Send for DmaFramebuffer {}
unsafe impl Sync for DmaFramebuffer {}

impl std::fmt::Debug for DmaFramebuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DmaFramebuffer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("byte_len", &self.buffer.len())
            .finish()
    }
}

impl DmaFramebuffer {
    pub fn as_ptr(&self) -> *mut u8 {
        self.buffer.as_ptr()
    }

    pub fn byte_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn phys_addr(&self) -> usize {
        self.buffer.phys_addr()
    }
}

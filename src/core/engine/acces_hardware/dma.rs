pub fn alloc_dma_framebuffer(width: usize, height: usize) -> Option<DmaFramebuffer> {
    let pixel_bytes = width.checked_mul(height)?.checked_mul(3)?.checked_mul(8)?;
    let buf = vec![0_u8; pixel_bytes];
    Some(DmaFramebuffer {
        width,
        height,
        buffer: buf,
    })
}

pub struct DmaFramebuffer {
    pub width: usize,
    pub height: usize,
    buffer: Vec<u8>,
}

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
        self.buffer.as_ptr() as *mut u8
    }

    pub fn byte_len(&self) -> usize {
        self.buffer.len()
    }

    pub fn virt_addr(&self) -> usize {
        self.buffer.as_ptr() as usize
    }
}

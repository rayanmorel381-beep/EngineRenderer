//! Win32 + WGL backend (currently a stub returning `None`).

use super::super::{BackendEvent, WindowBackend};

pub struct WindowsWindow {
    w: u32,
    h: u32,
}

impl WindowBackend for WindowsWindow {
    fn open(_w: u32, _h: u32, _t: &str) -> Option<Self> {
        None
    }
    fn width(&self) -> u32 {
        self.w
    }
    fn height(&self) -> u32 {
        self.h
    }
    fn should_close(&self) -> bool {
        true
    }
    fn pump_events(&mut self) -> Vec<BackendEvent> {
        Vec::new()
    }
    fn make_current(&self) -> bool {
        false
    }
    fn swap_buffers(&self) {}
    fn get_proc_address(&self, _n: &[u8]) -> *mut core::ffi::c_void {
        core::ptr::null_mut()
    }
}

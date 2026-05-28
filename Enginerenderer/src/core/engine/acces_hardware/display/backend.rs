//! Backend trait and per-OS type alias used by [`super::NativeWindow`].

use super::event::BackendEvent;

/// Trait implemented by every OS backend.
pub(crate) trait WindowBackend: Sized {
    fn open(width: u32, height: u32, title: &str) -> Option<Self>;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn should_close(&self) -> bool;
    fn pump_events(&mut self) -> Vec<BackendEvent>;
    fn make_current(&self) -> bool;
    fn swap_buffers(&self);
    fn get_proc_address(&self, name: &[u8]) -> *mut core::ffi::c_void;
    fn take_dropped_files(&mut self) -> Vec<String> {
        Vec::new()
    }
}

#[cfg(target_os = "linux")]
pub(crate) type Backend = super::linux::LinuxWindow;

#[cfg(target_os = "windows")]
pub(crate) type Backend = super::windows::WindowsWindow;

#[cfg(target_os = "macos")]
pub(crate) type Backend = super::macos::MacosWindow;

#[cfg(target_os = "android")]
pub(crate) type Backend = super::android::AndroidWindow;

#[cfg(not(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_os = "android"
)))]
pub(crate) type Backend = StubWindow;

#[cfg(not(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_os = "android"
)))]
#[derive(Debug)]
pub struct StubWindow {
    w: u32,
    h: u32,
}

#[cfg(not(any(
    target_os = "linux",
    target_os = "windows",
    target_os = "macos",
    target_os = "android"
)))]
impl WindowBackend for StubWindow {
    fn open(w: u32, h: u32, _t: &str) -> Option<Self> {
        Some(Self { w, h })
    }
    fn width(&self) -> u32 {
        self.w
    }
    fn height(&self) -> u32 {
        self.h
    }
    fn should_close(&self) -> bool {
        false
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

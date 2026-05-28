use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;

use crate::platform::launch;

#[cfg(target_os = "ios")]
#[link(name = "UIKit", kind = "framework")]
unsafe extern "C" {
    #[link_name = "UIApplicationMain"]
    fn ui_application_main(
        argc: c_int,
        argv: *mut *mut c_char,
        principal_class_name: *const c_void,
        delegate_class_name: *const c_void,
    ) -> c_int;
}

#[cfg(not(target_os = "ios"))]
unsafe fn ui_application_main(
    _argc: c_int,
    _argv: *mut *mut c_char,
    _principal_class_name: *const c_void,
    _delegate_class_name: *const c_void,
) -> c_int {
    launch() as c_int
}

static RUNNER: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);
static FRAME_TICK_REQUESTED: AtomicBool = AtomicBool::new(false);

fn spawn_runner() {
    if let Ok(mut guard) = RUNNER.lock()
        && guard.is_none()
    {
        let handle = thread::Builder::new()
            .name("enginerenderer-ios".into())
            .spawn(|| {
                let _ = launch();
            })
            .ok();
        *guard = handle;
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ruxel_app_entry() -> c_int {
    spawn_runner();
    unsafe { ui_application_main(0, ptr::null_mut(), ptr::null(), ptr::null()) }
}

#[unsafe(no_mangle)]
pub extern "C" fn ruxel_app_display_link_tick() {
    FRAME_TICK_REQUESTED.store(true, Ordering::SeqCst);
}

#[unsafe(no_mangle)]
pub extern "C" fn ruxel_app_consume_tick() -> c_int {
    if FRAME_TICK_REQUESTED.swap(false, Ordering::SeqCst) {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ruxel_app_run_blocking() -> c_int {
    launch() as c_int
}

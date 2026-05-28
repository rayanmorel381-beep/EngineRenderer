use core::ffi::{c_char, c_int, c_void};
use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::thread;

use crate::platform::launch;

#[repr(C)]
pub struct ANativeActivityCallbacks {
    pub on_start: Option<extern "C" fn(*mut ANativeActivity)>,
    pub on_resume: Option<extern "C" fn(*mut ANativeActivity)>,
    pub on_save_instance_state:
        Option<extern "C" fn(*mut ANativeActivity, *mut usize) -> *mut c_void>,
    pub on_pause: Option<extern "C" fn(*mut ANativeActivity)>,
    pub on_stop: Option<extern "C" fn(*mut ANativeActivity)>,
    pub on_destroy: Option<extern "C" fn(*mut ANativeActivity)>,
    pub on_window_focus_changed: Option<extern "C" fn(*mut ANativeActivity, c_int)>,
    pub on_native_window_created: Option<extern "C" fn(*mut ANativeActivity, *mut c_void)>,
    pub on_native_window_resized: Option<extern "C" fn(*mut ANativeActivity, *mut c_void)>,
    pub on_native_window_redraw_needed: Option<extern "C" fn(*mut ANativeActivity, *mut c_void)>,
    pub on_native_window_destroyed: Option<extern "C" fn(*mut ANativeActivity, *mut c_void)>,
    pub on_input_queue_created: Option<extern "C" fn(*mut ANativeActivity, *mut c_void)>,
    pub on_input_queue_destroyed: Option<extern "C" fn(*mut ANativeActivity, *mut c_void)>,
    pub on_content_rect_changed: Option<extern "C" fn(*mut ANativeActivity, *const c_void)>,
    pub on_configuration_changed: Option<extern "C" fn(*mut ANativeActivity)>,
    pub on_low_memory: Option<extern "C" fn(*mut ANativeActivity)>,
}

#[repr(C)]
pub struct ANativeActivity {
    pub callbacks: *mut ANativeActivityCallbacks,
    pub vm: *mut c_void,
    pub env: *mut c_void,
    pub clazz: *mut c_void,
    pub internal_data_path: *const c_char,
    pub external_data_path: *const c_char,
    pub sdk_version: i32,
    pub instance: *mut c_void,
    pub asset_manager: *mut c_void,
    pub obb_path: *const c_char,
}

static SHOULD_QUIT: AtomicBool = AtomicBool::new(false);
static RUNNER: Mutex<Option<thread::JoinHandle<()>>> = Mutex::new(None);

extern "C" fn cb_destroy(_activity: *mut ANativeActivity) {
    SHOULD_QUIT.store(true, Ordering::SeqCst);
    if let Ok(mut guard) = RUNNER.lock()
        && let Some(handle) = guard.take()
    {
        let _ = handle.join();
    }
}

extern "C" fn cb_focus(_activity: *mut ANativeActivity, _focus: c_int) {}
extern "C" fn cb_window(_activity: *mut ANativeActivity, _window: *mut c_void) {}
extern "C" fn cb_input(_activity: *mut ANativeActivity, _queue: *mut c_void) {}
extern "C" fn cb_lifecycle(_activity: *mut ANativeActivity) {}
extern "C" fn cb_content_rect(_activity: *mut ANativeActivity, _rect: *const c_void) {}
extern "C" fn cb_save_state(
    _activity: *mut ANativeActivity,
    out_size: *mut usize,
) -> *mut c_void {
    if !out_size.is_null() {
        unsafe { *out_size = 0 };
    }
    core::ptr::null_mut()
}

fn install_callbacks(callbacks: &mut ANativeActivityCallbacks) {
    callbacks.on_start = Some(cb_lifecycle);
    callbacks.on_resume = Some(cb_lifecycle);
    callbacks.on_pause = Some(cb_lifecycle);
    callbacks.on_stop = Some(cb_lifecycle);
    callbacks.on_destroy = Some(cb_destroy);
    callbacks.on_window_focus_changed = Some(cb_focus);
    callbacks.on_native_window_created = Some(cb_window);
    callbacks.on_native_window_resized = Some(cb_window);
    callbacks.on_native_window_redraw_needed = Some(cb_window);
    callbacks.on_native_window_destroyed = Some(cb_window);
    callbacks.on_input_queue_created = Some(cb_input);
    callbacks.on_input_queue_destroyed = Some(cb_input);
    callbacks.on_content_rect_changed = Some(cb_content_rect);
    callbacks.on_configuration_changed = Some(cb_lifecycle);
    callbacks.on_low_memory = Some(cb_lifecycle);
    callbacks.on_save_instance_state = Some(cb_save_state);
}

fn spawn_runner() {
    if let Ok(mut guard) = RUNNER.lock()
        && guard.is_none()
    {
        let handle = thread::Builder::new()
            .name("enginerenderer-android".into())
            .spawn(|| {
                let _ = launch();
            })
            .ok();
        *guard = handle;
    }
}

/// # Safety
///
/// Invoked by the Android `NativeActivity` runtime. `activity` must be a valid pointer to an
/// `ANativeActivity` whose `callbacks` field is writable for the lifetime of this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ANativeActivity_onCreate(
    activity: *mut ANativeActivity,
    _saved_state: *mut c_void,
    _saved_state_size: usize,
) {
    if activity.is_null() {
        return;
    }
    unsafe {
        if let Some(callbacks) = (*activity).callbacks.as_mut() {
            install_callbacks(callbacks);
        }
    }
    SHOULD_QUIT.store(false, Ordering::SeqCst);
    spawn_runner();
}

#[unsafe(no_mangle)]
pub extern "C" fn android_main(_activity: *mut c_void) -> c_int {
    launch() as c_int
}

#[unsafe(no_mangle)]
pub extern "C" fn ruxel_app_entry() -> c_int {
    launch() as c_int
}

pub fn should_quit() -> bool {
    SHOULD_QUIT.load(Ordering::SeqCst)
}

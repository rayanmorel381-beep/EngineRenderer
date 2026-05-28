//! Android GPU backend integration tests.
//!
//! These tests exercise the EGL + GLES3 pbuffer path of `NativeWindow`. They
//! only run when the target OS is Android (i.e. cross-compiled with
//! `--target aarch64-linux-android` and executed via `adb shell`).
//!
//! On any other OS the tests are skipped at compile time, so a regular
//! `cargo test` on the desktop is unaffected.

#![cfg(target_os = "android")]

use core::ffi::{c_char, c_uint, c_void};

use enginerenderer::api::display::NativeWindow;

const GL_NO_ERROR: c_uint = 0;
const GL_COLOR_BUFFER_BIT: c_uint = 0x0000_4000;
const GL_VENDOR: c_uint = 0x1F00;
const GL_RENDERER: c_uint = 0x1F01;
const GL_VERSION: c_uint = 0x1F02;

type GlClearColor = unsafe extern "C" fn(f32, f32, f32, f32);
type GlClear = unsafe extern "C" fn(c_uint);
type GlGetString = unsafe extern "C" fn(c_uint) -> *const c_char;
type GlGetError = unsafe extern "C" fn() -> c_uint;
type GlViewport = unsafe extern "C" fn(i32, i32, i32, i32);

unsafe fn load<T: Copy>(window: &NativeWindow, name: &[u8]) -> T {
    let p = window.gl_get_proc(name);
    assert!(
        !p.is_null(),
        "GL function {:?} not found",
        core::str::from_utf8(name).unwrap_or("?")
    );
    assert_eq!(
        core::mem::size_of::<T>(),
        core::mem::size_of::<*mut c_void>(),
        "function pointer size mismatch"
    );
    unsafe { core::mem::transmute_copy(&p) }
}

fn read_cstring(p: *const c_char) -> String {
    if p.is_null() {
        return String::new();
    }
    let mut len = 0usize;
    while unsafe { *p.add(len) } != 0 {
        len += 1;
        if len > 4096 {
            break;
        }
    }
    let bytes = unsafe { core::slice::from_raw_parts(p as *const u8, len) };
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn opens_window_and_attaches_backend() {
    let window = NativeWindow::open(64, 64, "test").expect("open returned None");
    assert!(
        window.has_backend(),
        "EGL backend should be attached on Android"
    );
    assert_eq!(window.width, 64);
    assert_eq!(window.height, 64);
    assert!(window.vsync_capacity >= 1);
}

#[test]
fn make_current_succeeds() {
    let window = NativeWindow::open(64, 64, "test").expect("open returned None");
    assert!(window.make_current(), "eglMakeCurrent should succeed");
}

#[test]
fn resolves_core_gles_functions() {
    let window = NativeWindow::open(64, 64, "test").expect("open returned None");
    assert!(window.make_current());
    for name in [
        &b"glClear\0"[..],
        &b"glClearColor\0"[..],
        &b"glGetString\0"[..],
        &b"glGetError\0"[..],
        &b"glViewport\0"[..],
    ] {
        let p = window.gl_get_proc(name);
        assert!(
            !p.is_null(),
            "GLES function {:?} could not be resolved",
            core::str::from_utf8(name).unwrap()
        );
    }
}

#[test]
fn reports_opengl_es_strings() {
    let window = NativeWindow::open(64, 64, "test").expect("open returned None");
    assert!(window.make_current());
    let gl_get_string: GlGetString = unsafe { load(&window, b"glGetString\0") };
    let vendor = read_cstring(unsafe { gl_get_string(GL_VENDOR) });
    let renderer = read_cstring(unsafe { gl_get_string(GL_RENDERER) });
    let version = read_cstring(unsafe { gl_get_string(GL_VERSION) });
    assert!(!vendor.is_empty(), "GL_VENDOR is empty");
    assert!(!renderer.is_empty(), "GL_RENDERER is empty");
    assert!(
        version.contains("OpenGL ES"),
        "GL_VERSION does not contain 'OpenGL ES': {version:?}"
    );
}

#[test]
fn full_frame_clear_and_swap_succeeds() {
    let window = NativeWindow::open(128, 96, "test").expect("open returned None");
    assert!(window.make_current());
    let gl_viewport: GlViewport = unsafe { load(&window, b"glViewport\0") };
    let gl_clear_color: GlClearColor = unsafe { load(&window, b"glClearColor\0") };
    let gl_clear: GlClear = unsafe { load(&window, b"glClear\0") };
    let gl_get_error: GlGetError = unsafe { load(&window, b"glGetError\0") };
    unsafe {
        gl_viewport(0, 0, window.width as i32, window.height as i32);
        gl_clear_color(0.25, 0.5, 0.75, 1.0);
        gl_clear(GL_COLOR_BUFFER_BIT);
        assert_eq!(gl_get_error(), GL_NO_ERROR, "GL error after clear");
    }
    window.swap_buffers();
}

#[test]
fn open_then_drop_repeatedly_does_not_leak_or_crash() {
    for _ in 0..8 {
        let window = NativeWindow::open(32, 32, "test").expect("open returned None");
        assert!(window.make_current());
        window.swap_buffers();
        drop(window);
    }
}

#[test]
fn vsync_capacity_is_within_documented_bounds() {
    let window = NativeWindow::open(800, 600, "test").expect("open returned None");
    assert!(window.vsync_capacity >= 1);
    assert!(
        window.vsync_capacity <= 4,
        "vsync_capacity={} exceeds expected bound (max 4)",
        window.vsync_capacity
    );
}

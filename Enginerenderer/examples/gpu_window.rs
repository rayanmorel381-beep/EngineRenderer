//! Opens a real OS window with an OpenGL 3.3 core context attached, queries
//! the GL renderer / version strings, then runs a 120-frame loop clearing to
//! a moving colour and presenting via swap-buffers.
//!
//! Run with:
//!     cargo run --example gpu_window
//!
//! Press the close button or wait 120 frames to exit.

use core::ffi::{c_char, c_uint, c_void};

use enginerenderer::api::display::NativeWindow;

type GlClearColor = unsafe extern "C" fn(f32, f32, f32, f32);
type GlClear = unsafe extern "C" fn(c_uint);
type GlViewport = unsafe extern "C" fn(i32, i32, i32, i32);
type GlGetString = unsafe extern "C" fn(c_uint) -> *const c_char;

const GL_COLOR_BUFFER_BIT: c_uint = 0x0000_4000;
const GL_DEPTH_BUFFER_BIT: c_uint = 0x0000_0100;
const GL_VENDOR: c_uint = 0x1F00;
const GL_RENDERER: c_uint = 0x1F01;
const GL_VERSION: c_uint = 0x1F02;

fn load<T: Copy>(window: &NativeWindow, name: &[u8]) -> Option<T> {
    let p = window.gl_get_proc(name);
    if p.is_null() {
        return None;
    }
    assert_eq!(
        core::mem::size_of::<T>(),
        core::mem::size_of::<*mut c_void>(),
        "function pointer size mismatch"
    );
    let f: T = unsafe { core::mem::transmute_copy(&p) };
    Some(f)
}

fn read_cstring(p: *const c_char) -> String {
    if p.is_null() {
        return String::from("<null>");
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

fn main() {
    let mut window = match NativeWindow::open(800, 600, "EngineRenderer · gpu_window") {
        Some(w) => w,
        None => {
            eprintln!("failed to open native window");
            std::process::exit(1);
        }
    };

    if !window.has_backend() {
        println!(
            "[gpu_window] no GPU backend available on this platform — window is logical only ({}x{}).",
            window.width, window.height
        );
        println!("[gpu_window] vsync_capacity = {}", window.vsync_capacity);
        return;
    }

    if !window.make_current() {
        eprintln!("failed to make GL context current");
        std::process::exit(1);
    }

    let gl_clear_color: GlClearColor =
        load(&window, b"glClearColor\0").expect("glClearColor not found");
    let gl_clear: GlClear = load(&window, b"glClear\0").expect("glClear not found");
    let gl_viewport: GlViewport = load(&window, b"glViewport\0").expect("glViewport not found");
    let gl_get_string: GlGetString =
        load(&window, b"glGetString\0").expect("glGetString not found");

    let vendor = read_cstring(unsafe { gl_get_string(GL_VENDOR) });
    let renderer = read_cstring(unsafe { gl_get_string(GL_RENDERER) });
    let version = read_cstring(unsafe { gl_get_string(GL_VERSION) });

    println!("[gpu_window] GL_VENDOR   = {vendor}");
    println!("[gpu_window] GL_RENDERER = {renderer}");
    println!("[gpu_window] GL_VERSION  = {version}");
    println!(
        "[gpu_window] window      = {}x{} (vsync_capacity = {})",
        window.width, window.height, window.vsync_capacity
    );

    let max_frames = 120u32;
    let mut frame = 0u32;
    while !window.should_close() && frame < max_frames {
        window.pump();
        unsafe {
            gl_viewport(0, 0, window.width as i32, window.height as i32);
            let t = frame as f32 / max_frames as f32;
            let r = 0.2 + 0.6 * t;
            let g = 0.3 + 0.4 * (1.0 - t);
            let b = 0.6;
            gl_clear_color(r, g, b, 1.0);
            gl_clear(GL_COLOR_BUFFER_BIT | GL_DEPTH_BUFFER_BIT);
        }
        window.swap_buffers();
        frame = frame.saturating_add(1);
    }

    println!("[gpu_window] presented {frame} frames, exiting.");
}

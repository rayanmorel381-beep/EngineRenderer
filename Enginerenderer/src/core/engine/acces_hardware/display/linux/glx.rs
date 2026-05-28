//! GLX context creation (OpenGL binding via X11).
//!
//! OS-specific (Linux/X11), vendor-agnostic. The driver implementation behind
//! `libGL.so` may come from Mesa (amdgpu/i915/nouveau) or a vendor blob, but
//! the GLX entry points we use here are the same in every case.

use core::ffi::{c_char, c_int, c_void};
use core::ptr;
use std::sync::Mutex;

use super::x11::{xlib, Display, XVisualInfo, RTLD_GLOBAL, RTLD_NOW};

pub(crate) type GLXFBConfig = *mut c_void;
pub(crate) type GLXContext = *mut c_void;
pub(crate) type GLXDrawable = core::ffi::c_ulong;

pub(crate) const GLX_RGBA_BIT: c_int = 0x00000001;
pub(crate) const GLX_RENDER_TYPE: c_int = 0x8011;
pub(crate) const GLX_DRAWABLE_TYPE: c_int = 0x8010;
pub(crate) const GLX_WINDOW_BIT: c_int = 0x00000001;
pub(crate) const GLX_PBUFFER_BIT: c_int = 0x00000004;
pub(crate) const GLX_PBUFFER_WIDTH: c_int = 0x8041;
pub(crate) const GLX_PBUFFER_HEIGHT: c_int = 0x8040;
pub(crate) const GLX_X_VISUAL_TYPE: c_int = 0x22;
pub(crate) const GLX_TRUE_COLOR: c_int = 0x8002;
pub(crate) const GLX_RED_SIZE: c_int = 8;
pub(crate) const GLX_GREEN_SIZE: c_int = 9;
pub(crate) const GLX_BLUE_SIZE: c_int = 10;
pub(crate) const GLX_ALPHA_SIZE: c_int = 11;
pub(crate) const GLX_DEPTH_SIZE: c_int = 12;
pub(crate) const GLX_STENCIL_SIZE: c_int = 13;
pub(crate) const GLX_DOUBLEBUFFER: c_int = 5;
pub(crate) const GLX_CONTEXT_MAJOR_VERSION_ARB: c_int = 0x2091;
pub(crate) const GLX_CONTEXT_MINOR_VERSION_ARB: c_int = 0x2092;
pub(crate) const GLX_CONTEXT_PROFILE_MASK_ARB: c_int = 0x9126;
pub(crate) const GLX_CONTEXT_CORE_PROFILE_BIT_ARB: c_int = 0x00000001;
pub(crate) const GLX_CONTEXT_FLAGS_ARB: c_int = 0x2094;
pub(crate) const GLX_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB: c_int = 0x00000002;

unsafe extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
}

type FnGlxChooseFBConfig =
    unsafe extern "C" fn(*mut Display, c_int, *const c_int, *mut c_int) -> *mut GLXFBConfig;
type FnGlxGetVisualFromFBConfig =
    unsafe extern "C" fn(*mut Display, GLXFBConfig) -> *mut XVisualInfo;
type FnGlxCreateNewContext =
    unsafe extern "C" fn(*mut Display, GLXFBConfig, c_int, GLXContext, c_int) -> GLXContext;
type FnGlxDestroyContext = unsafe extern "C" fn(*mut Display, GLXContext);
type FnGlxMakeCurrent = unsafe extern "C" fn(*mut Display, GLXDrawable, GLXContext) -> c_int;
type FnGlxSwapBuffers = unsafe extern "C" fn(*mut Display, GLXDrawable);
type FnGlxGetProcAddress = unsafe extern "C" fn(*const c_char) -> *mut c_void;
type FnGlxQueryVersion = unsafe extern "C" fn(*mut Display, *mut c_int, *mut c_int) -> c_int;
type FnGlxCreateContextAttribsARB =
    unsafe extern "C" fn(*mut Display, GLXFBConfig, GLXContext, c_int, *const c_int) -> GLXContext;
type FnGlxCreatePbuffer =
    unsafe extern "C" fn(*mut Display, GLXFBConfig, *const c_int) -> GLXDrawable;
type FnGlxDestroyPbuffer = unsafe extern "C" fn(*mut Display, GLXDrawable);
type FnGlxSwapIntervalEXT = unsafe extern "C" fn(*mut Display, GLXDrawable, c_int);

#[allow(non_snake_case, dead_code)]
pub(crate) struct GlxLib {
    pub handle: *mut c_void,
    pub glXChooseFBConfig: FnGlxChooseFBConfig,
    pub glXGetVisualFromFBConfig: FnGlxGetVisualFromFBConfig,
    pub glXCreateNewContext: FnGlxCreateNewContext,
    pub glXDestroyContext: FnGlxDestroyContext,
    pub glXMakeCurrent: FnGlxMakeCurrent,
    pub glXSwapBuffers: FnGlxSwapBuffers,
    pub glXGetProcAddress: FnGlxGetProcAddress,
    pub glXQueryVersion: FnGlxQueryVersion,
    pub glXCreateContextAttribsARB: Option<FnGlxCreateContextAttribsARB>,
    pub glXCreatePbuffer: Option<FnGlxCreatePbuffer>,
    pub glXDestroyPbuffer: Option<FnGlxDestroyPbuffer>,
    pub glXSwapIntervalEXT: Option<FnGlxSwapIntervalEXT>,
}

unsafe impl Send for GlxLib {}
unsafe impl Sync for GlxLib {}

static GLX: Mutex<Option<&'static GlxLib>> = Mutex::new(None);

unsafe fn load_sym<T: Copy>(handle: *mut c_void, name: &[u8]) -> Option<T> {
    debug_assert_eq!(name.last(), Some(&0));
    let sym = unsafe { dlsym(handle, name.as_ptr() as *const c_char) };
    if sym.is_null() {
        return None;
    }
    Some(unsafe { core::mem::transmute_copy::<*mut c_void, T>(&sym) })
}

fn try_open(name: &[u8]) -> Option<*mut c_void> {
    let handle = unsafe { dlopen(name.as_ptr() as *const c_char, RTLD_NOW | RTLD_GLOBAL) };
    if handle.is_null() {
        None
    } else {
        Some(handle)
    }
}

pub(crate) fn glx() -> Option<&'static GlxLib> {
    let mut guard = GLX.lock().ok()?;
    if let Some(lib) = *guard {
        return Some(lib);
    }
    let handle = try_open(b"libGL.so.1\0").or_else(|| try_open(b"libGL.so\0"))?;
    let glx_get_proc_address: FnGlxGetProcAddress =
        unsafe { load_sym(handle, b"glXGetProcAddress\0") }
            .or_else(|| unsafe { load_sym(handle, b"glXGetProcAddressARB\0") })?;

    fn arb<T: Copy>(get: FnGlxGetProcAddress, name: &[u8]) -> Option<T> {
        let p = unsafe { get(name.as_ptr() as *const c_char) };
        if p.is_null() {
            None
        } else {
            Some(unsafe { core::mem::transmute_copy::<*mut c_void, T>(&p) })
        }
    }

    let lib = unsafe {
        GlxLib {
            handle,
            glXChooseFBConfig: load_sym(handle, b"glXChooseFBConfig\0")?,
            glXGetVisualFromFBConfig: load_sym(handle, b"glXGetVisualFromFBConfig\0")?,
            glXCreateNewContext: load_sym(handle, b"glXCreateNewContext\0")?,
            glXDestroyContext: load_sym(handle, b"glXDestroyContext\0")?,
            glXMakeCurrent: load_sym(handle, b"glXMakeCurrent\0")?,
            glXSwapBuffers: load_sym(handle, b"glXSwapBuffers\0")?,
            glXGetProcAddress: glx_get_proc_address,
            glXQueryVersion: load_sym(handle, b"glXQueryVersion\0")?,
            glXCreateContextAttribsARB: arb(
                glx_get_proc_address,
                b"glXCreateContextAttribsARB\0",
            ),
            glXCreatePbuffer: load_sym(handle, b"glXCreatePbuffer\0"),
            glXDestroyPbuffer: load_sym(handle, b"glXDestroyPbuffer\0"),
            glXSwapIntervalEXT: arb(glx_get_proc_address, b"glXSwapIntervalEXT\0"),
        }
    };
    let leaked: &'static GlxLib = Box::leak(Box::new(lib));
    *guard = Some(leaked);
    Some(leaked)
}

#[allow(dead_code)]
pub(crate) struct GlxContext {
    pub display: *mut Display,
    pub context: GLXContext,
    pub drawable: GLXDrawable,
    pub fb_config: GLXFBConfig,
    pub visual_info: XVisualInfo,
}

unsafe impl Send for GlxContext {}
unsafe impl Sync for GlxContext {}

pub(crate) fn choose_visual_only(
    display: *mut Display,
    screen: c_int,
) -> Option<(XVisualInfo, GLXFBConfig)> {
    let lib = glx()?;
    let attrs: [c_int; 21] = [
        GLX_X_VISUAL_TYPE,
        GLX_TRUE_COLOR,
        GLX_RENDER_TYPE,
        GLX_RGBA_BIT,
        GLX_DRAWABLE_TYPE,
        GLX_WINDOW_BIT,
        GLX_RED_SIZE,
        8,
        GLX_GREEN_SIZE,
        8,
        GLX_BLUE_SIZE,
        8,
        GLX_ALPHA_SIZE,
        8,
        GLX_DEPTH_SIZE,
        24,
        GLX_STENCIL_SIZE,
        8,
        GLX_DOUBLEBUFFER,
        1,
        0,
    ];
    let mut count: c_int = 0;
    let configs = unsafe { (lib.glXChooseFBConfig)(display, screen, attrs.as_ptr(), &mut count) };
    if configs.is_null() || count <= 0 {
        return None;
    }
    let chosen = unsafe { *configs };
    let vi_ptr = unsafe { (lib.glXGetVisualFromFBConfig)(display, chosen) };
    if vi_ptr.is_null() {
        let xlib = xlib()?;
        unsafe { (xlib.XFree)(configs as *mut c_void) };
        return None;
    }
    let vi = unsafe { ptr::read(vi_ptr) };
    let xlib = xlib()?;
    unsafe {
        (xlib.XFree)(vi_ptr as *mut c_void);
        (xlib.XFree)(configs as *mut c_void);
    }
    Some((vi, chosen))
}

pub(crate) fn create_context(
    display: *mut Display,
    fb_config: GLXFBConfig,
    drawable: GLXDrawable,
    visual_info: XVisualInfo,
) -> Option<GlxContext> {
    let lib = glx()?;
    let context = if let Some(ctx_attribs) = lib.glXCreateContextAttribsARB {
        let attrs: [c_int; 9] = [
            GLX_CONTEXT_MAJOR_VERSION_ARB,
            3,
            GLX_CONTEXT_MINOR_VERSION_ARB,
            3,
            GLX_CONTEXT_PROFILE_MASK_ARB,
            GLX_CONTEXT_CORE_PROFILE_BIT_ARB,
            GLX_CONTEXT_FLAGS_ARB,
            GLX_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
            0,
        ];
        unsafe { ctx_attribs(display, fb_config, ptr::null_mut(), 1, attrs.as_ptr()) }
    } else {
        unsafe { (lib.glXCreateNewContext)(display, fb_config, GLX_RGBA_BIT, ptr::null_mut(), 1) }
    };
    if context.is_null() {
        return None;
    }
    let ok = unsafe { (lib.glXMakeCurrent)(display, drawable, context) };
    if ok == 0 {
        unsafe { (lib.glXDestroyContext)(display, context) };
        return None;
    }
    Some(GlxContext {
        display,
        context,
        drawable,
        fb_config,
        visual_info,
    })
}

impl GlxContext {
    pub fn make_current(&self) -> bool {
        let Some(lib) = glx() else {
            return false;
        };
        unsafe { (lib.glXMakeCurrent)(self.display, self.drawable, self.context) != 0 }
    }

    pub fn swap_buffers(&self) {
        if let Some(lib) = glx() {
            unsafe { (lib.glXSwapBuffers)(self.display, self.drawable) };
        }
    }

    pub fn set_swap_interval(&self, interval: i32) {
        if let Some(lib) = glx()
            && let Some(set) = lib.glXSwapIntervalEXT
        {
            unsafe { set(self.display, self.drawable, interval as c_int) };
        }
    }

    pub fn get_proc(&self, name: &[u8]) -> *mut c_void {
        debug_assert_eq!(name.last(), Some(&0));
        let Some(lib) = glx() else {
            return ptr::null_mut();
        };
        unsafe { (lib.glXGetProcAddress)(name.as_ptr() as *const c_char) }
    }
}

impl Drop for GlxContext {
    fn drop(&mut self) {
        if let Some(lib) = glx() {
            unsafe {
                (lib.glXMakeCurrent)(self.display, 0, ptr::null_mut());
                if !self.context.is_null() {
                    (lib.glXDestroyContext)(self.display, self.context);
                    self.context = ptr::null_mut();
                }
            }
        }
    }
}

/// Offscreen GLX context built around a `GLXPbuffer`. Owns its X11 `Display`
/// connection so it can be used standalone, without any visible window.
pub(crate) struct OffscreenContext {
    pub display: *mut Display,
    pub context: GLXContext,
    pub pbuffer: GLXDrawable,
    pub width: u32,
    pub height: u32,
}

unsafe impl Send for OffscreenContext {}
unsafe impl Sync for OffscreenContext {}

/// Opens a fresh X11 display, picks a Pbuffer-capable FBConfig, creates a
/// GL `major.minor` core context, and binds it to a Pbuffer of the requested
/// size. Returns `None` if any step fails (no X server, no GL ≥ requested
/// version, no Pbuffer support, etc.).
pub(crate) fn create_offscreen_context(
    width: u32,
    height: u32,
    major: i32,
    minor: i32,
) -> Option<OffscreenContext> {
    let xlib = xlib()?;
    let lib = glx()?;
    let create_pbuffer = lib.glXCreatePbuffer?;
    let create_attribs = lib.glXCreateContextAttribsARB?;

    let display = unsafe { (xlib.XOpenDisplay)(ptr::null()) };
    if display.is_null() {
        return None;
    }
    let screen = unsafe { (xlib.XDefaultScreen)(display) };

    let attrs: [c_int; 17] = [
        GLX_RENDER_TYPE,
        GLX_RGBA_BIT,
        GLX_DRAWABLE_TYPE,
        GLX_PBUFFER_BIT,
        GLX_RED_SIZE,
        8,
        GLX_GREEN_SIZE,
        8,
        GLX_BLUE_SIZE,
        8,
        GLX_ALPHA_SIZE,
        8,
        GLX_DEPTH_SIZE,
        24,
        GLX_DOUBLEBUFFER,
        0,
        0,
    ];
    let mut count: c_int = 0;
    let configs = unsafe { (lib.glXChooseFBConfig)(display, screen, attrs.as_ptr(), &mut count) };
    if configs.is_null() || count <= 0 {
        unsafe { (xlib.XCloseDisplay)(display) };
        return None;
    }
    let fb_config = unsafe { *configs };
    unsafe { (xlib.XFree)(configs as *mut c_void) };

    let pb_attrs: [c_int; 5] = [
        GLX_PBUFFER_WIDTH,
        width as c_int,
        GLX_PBUFFER_HEIGHT,
        height as c_int,
        0,
    ];
    let pbuffer = unsafe { create_pbuffer(display, fb_config, pb_attrs.as_ptr()) };
    if pbuffer == 0 {
        unsafe { (xlib.XCloseDisplay)(display) };
        return None;
    }

    let ctx_attrs: [c_int; 9] = [
        GLX_CONTEXT_MAJOR_VERSION_ARB,
        major,
        GLX_CONTEXT_MINOR_VERSION_ARB,
        minor,
        GLX_CONTEXT_PROFILE_MASK_ARB,
        GLX_CONTEXT_CORE_PROFILE_BIT_ARB,
        GLX_CONTEXT_FLAGS_ARB,
        GLX_CONTEXT_FORWARD_COMPATIBLE_BIT_ARB,
        0,
    ];
    let context = unsafe { create_attribs(display, fb_config, ptr::null_mut(), 1, ctx_attrs.as_ptr()) };
    if context.is_null() {
        if let Some(destroy) = lib.glXDestroyPbuffer {
            unsafe { destroy(display, pbuffer) };
        }
        unsafe { (xlib.XCloseDisplay)(display) };
        return None;
    }

    let ok = unsafe { (lib.glXMakeCurrent)(display, pbuffer, context) };
    if ok == 0 {
        unsafe { (lib.glXDestroyContext)(display, context) };
        if let Some(destroy) = lib.glXDestroyPbuffer {
            unsafe { destroy(display, pbuffer) };
        }
        unsafe { (xlib.XCloseDisplay)(display) };
        return None;
    }

    Some(OffscreenContext {
        display,
        context,
        pbuffer,
        width,
        height,
    })
}

impl OffscreenContext {
    pub fn make_current(&self) -> bool {
        let Some(lib) = glx() else {
            return false;
        };
        unsafe { (lib.glXMakeCurrent)(self.display, self.pbuffer, self.context) != 0 }
    }

    pub fn get_proc(&self, name: &[u8]) -> *mut c_void {
        debug_assert_eq!(name.last(), Some(&0));
        let Some(lib) = glx() else {
            return ptr::null_mut();
        };
        unsafe { (lib.glXGetProcAddress)(name.as_ptr() as *const c_char) }
    }
}

impl Drop for OffscreenContext {
    fn drop(&mut self) {
        if let Some(lib) = glx() {
            unsafe { (lib.glXMakeCurrent)(self.display, 0, ptr::null_mut()) };
            if !self.context.is_null() {
                unsafe { (lib.glXDestroyContext)(self.display, self.context) };
                self.context = ptr::null_mut();
            }
            if self.pbuffer != 0
                && let Some(destroy) = lib.glXDestroyPbuffer
            {
                unsafe { destroy(self.display, self.pbuffer) };
                self.pbuffer = 0;
            }
        }
        if let Some(xlib) = xlib()
            && !self.display.is_null()
        {
            unsafe { (xlib.XCloseDisplay)(self.display) };
            self.display = ptr::null_mut();
        }
    }
}

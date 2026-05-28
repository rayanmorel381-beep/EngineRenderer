//! EGL FFI bindings (types, constants, function pointer table) and a lazily
//! initialised global handle to `libEGL.so` + `libGLESv*.so`.

use core::ffi::{c_char, c_int, c_uint, c_void};
use std::sync::Mutex;

use super::dl::{load_sym, try_open};

pub(super) type EGLDisplay = *mut c_void;
pub(super) type EGLConfig = *mut c_void;
pub(super) type EGLContext = *mut c_void;
pub(super) type EGLSurface = *mut c_void;
pub(super) type EGLNativeDisplayType = *mut c_void;
pub(super) type EGLBoolean = c_uint;

pub(super) const EGL_DEFAULT_DISPLAY: EGLNativeDisplayType = core::ptr::null_mut();
pub(super) const EGL_NO_CONTEXT: EGLContext = core::ptr::null_mut();
pub(super) const EGL_NO_SURFACE: EGLSurface = core::ptr::null_mut();
pub(super) const EGL_NO_DISPLAY: EGLDisplay = core::ptr::null_mut();

pub(super) const EGL_OPENGL_ES_API: c_uint = 0x30A0;

pub(super) const EGL_NONE: c_int = 0x3038;
pub(super) const EGL_HEIGHT: c_int = 0x3056;
pub(super) const EGL_WIDTH: c_int = 0x3057;
pub(super) const EGL_RED_SIZE: c_int = 0x3024;
pub(super) const EGL_GREEN_SIZE: c_int = 0x3023;
pub(super) const EGL_BLUE_SIZE: c_int = 0x3022;
pub(super) const EGL_ALPHA_SIZE: c_int = 0x3021;
pub(super) const EGL_DEPTH_SIZE: c_int = 0x3025;
pub(super) const EGL_STENCIL_SIZE: c_int = 0x3026;
pub(super) const EGL_SURFACE_TYPE: c_int = 0x3033;
pub(super) const EGL_PBUFFER_BIT: c_int = 0x0001;
pub(super) const EGL_RENDERABLE_TYPE: c_int = 0x3040;
pub(super) const EGL_OPENGL_ES3_BIT: c_int = 0x0040;
pub(super) const EGL_CONTEXT_CLIENT_VERSION: c_int = 0x3098;

pub(super) type FnEglGetDisplay = unsafe extern "C" fn(EGLNativeDisplayType) -> EGLDisplay;
pub(super) type FnEglInitialize =
    unsafe extern "C" fn(EGLDisplay, *mut c_int, *mut c_int) -> EGLBoolean;
pub(super) type FnEglTerminate = unsafe extern "C" fn(EGLDisplay) -> EGLBoolean;
pub(super) type FnEglBindAPI = unsafe extern "C" fn(c_uint) -> EGLBoolean;
pub(super) type FnEglChooseConfig =
    unsafe extern "C" fn(EGLDisplay, *const c_int, *mut EGLConfig, c_int, *mut c_int) -> EGLBoolean;
pub(super) type FnEglCreatePbufferSurface =
    unsafe extern "C" fn(EGLDisplay, EGLConfig, *const c_int) -> EGLSurface;
pub(super) type FnEglDestroySurface = unsafe extern "C" fn(EGLDisplay, EGLSurface) -> EGLBoolean;
pub(super) type FnEglCreateContext =
    unsafe extern "C" fn(EGLDisplay, EGLConfig, EGLContext, *const c_int) -> EGLContext;
pub(super) type FnEglDestroyContext = unsafe extern "C" fn(EGLDisplay, EGLContext) -> EGLBoolean;
pub(super) type FnEglMakeCurrent =
    unsafe extern "C" fn(EGLDisplay, EGLSurface, EGLSurface, EGLContext) -> EGLBoolean;
pub(super) type FnEglSwapBuffers = unsafe extern "C" fn(EGLDisplay, EGLSurface) -> EGLBoolean;
pub(super) type FnEglGetProcAddress = unsafe extern "C" fn(*const c_char) -> *mut c_void;
pub(super) type FnEglGetError = unsafe extern "C" fn() -> c_int;

#[allow(non_snake_case, dead_code)]
pub(super) struct EglLib {
    pub handle: *mut c_void,
    pub gles_handle: *mut c_void,
    pub eglGetDisplay: FnEglGetDisplay,
    pub eglInitialize: FnEglInitialize,
    pub eglTerminate: FnEglTerminate,
    pub eglBindAPI: FnEglBindAPI,
    pub eglChooseConfig: FnEglChooseConfig,
    pub eglCreatePbufferSurface: FnEglCreatePbufferSurface,
    pub eglDestroySurface: FnEglDestroySurface,
    pub eglCreateContext: FnEglCreateContext,
    pub eglDestroyContext: FnEglDestroyContext,
    pub eglMakeCurrent: FnEglMakeCurrent,
    pub eglSwapBuffers: FnEglSwapBuffers,
    pub eglGetProcAddress: FnEglGetProcAddress,
    pub eglGetError: FnEglGetError,
}

unsafe impl Send for EglLib {}
unsafe impl Sync for EglLib {}

static EGL: Mutex<Option<&'static EglLib>> = Mutex::new(None);

pub(super) fn egl() -> Option<&'static EglLib> {
    let mut guard = EGL.lock().ok()?;
    if let Some(lib) = *guard {
        return Some(lib);
    }
    let handle = try_open(b"libEGL.so\0")?;
    let gles_handle = try_open(b"libGLESv3.so\0")
        .or_else(|| try_open(b"libGLESv2.so\0"))
        .unwrap_or(core::ptr::null_mut());
    let lib = unsafe {
        EglLib {
            handle,
            gles_handle,
            eglGetDisplay: load_sym(handle, b"eglGetDisplay\0")?,
            eglInitialize: load_sym(handle, b"eglInitialize\0")?,
            eglTerminate: load_sym(handle, b"eglTerminate\0")?,
            eglBindAPI: load_sym(handle, b"eglBindAPI\0")?,
            eglChooseConfig: load_sym(handle, b"eglChooseConfig\0")?,
            eglCreatePbufferSurface: load_sym(handle, b"eglCreatePbufferSurface\0")?,
            eglDestroySurface: load_sym(handle, b"eglDestroySurface\0")?,
            eglCreateContext: load_sym(handle, b"eglCreateContext\0")?,
            eglDestroyContext: load_sym(handle, b"eglDestroyContext\0")?,
            eglMakeCurrent: load_sym(handle, b"eglMakeCurrent\0")?,
            eglSwapBuffers: load_sym(handle, b"eglSwapBuffers\0")?,
            eglGetProcAddress: load_sym(handle, b"eglGetProcAddress\0")?,
            eglGetError: load_sym(handle, b"eglGetError\0")?,
        }
    };
    let leaked: &'static EglLib = Box::leak(Box::new(lib));
    *guard = Some(leaked);
    Some(leaked)
}

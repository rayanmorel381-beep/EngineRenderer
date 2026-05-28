//! Android display backend (EGL + GLES3 offscreen pbuffer).
//!
//! No NativeActivity / Java dependency — opens an EGL `EGL_DEFAULT_DISPLAY`
//! and creates an off-screen pbuffer surface bound to a GLES 3 context.
//! Suitable for headless GPU compute, render-to-texture, and `adb shell`
//! invocation.

use core::ffi::{c_char, c_int, c_void};

use super::super::{BackendEvent, WindowBackend};
use super::dl::dlsym;
use super::egl::{
    EGL_ALPHA_SIZE, EGL_BLUE_SIZE, EGL_CONTEXT_CLIENT_VERSION, EGL_DEFAULT_DISPLAY, EGL_DEPTH_SIZE,
    EGL_GREEN_SIZE, EGL_HEIGHT, EGL_NO_CONTEXT, EGL_NO_DISPLAY, EGL_NO_SURFACE, EGL_NONE,
    EGL_OPENGL_ES_API, EGL_OPENGL_ES3_BIT, EGL_PBUFFER_BIT, EGL_RED_SIZE, EGL_RENDERABLE_TYPE,
    EGL_STENCIL_SIZE, EGL_SURFACE_TYPE, EGL_WIDTH, EGLConfig, EGLContext, EGLDisplay, EGLSurface,
    egl,
};

pub struct AndroidWindow {
    display: EGLDisplay,
    surface: EGLSurface,
    context: EGLContext,
    width: u32,
    height: u32,
}

impl WindowBackend for AndroidWindow {
    fn open(width: u32, height: u32, _title: &str) -> Option<Self> {
        let lib = egl()?;
        unsafe {
            let display = (lib.eglGetDisplay)(EGL_DEFAULT_DISPLAY);
            if display == EGL_NO_DISPLAY {
                return None;
            }
            let mut major = 0i32;
            let mut minor = 0i32;
            if (lib.eglInitialize)(display, &mut major, &mut minor) == 0 {
                return None;
            }
            if (lib.eglBindAPI)(EGL_OPENGL_ES_API) == 0 {
                (lib.eglTerminate)(display);
                return None;
            }
            let attrs = [
                EGL_SURFACE_TYPE,
                EGL_PBUFFER_BIT,
                EGL_RENDERABLE_TYPE,
                EGL_OPENGL_ES3_BIT,
                EGL_RED_SIZE,
                8,
                EGL_GREEN_SIZE,
                8,
                EGL_BLUE_SIZE,
                8,
                EGL_ALPHA_SIZE,
                8,
                EGL_DEPTH_SIZE,
                24,
                EGL_STENCIL_SIZE,
                8,
                EGL_NONE,
            ];
            let mut config: EGLConfig = core::ptr::null_mut();
            let mut num_configs = 0i32;
            if (lib.eglChooseConfig)(display, attrs.as_ptr(), &mut config, 1, &mut num_configs) == 0
                || num_configs == 0
            {
                (lib.eglTerminate)(display);
                return None;
            }
            let surface_attrs = [
                EGL_WIDTH,
                width as c_int,
                EGL_HEIGHT,
                height as c_int,
                EGL_NONE,
            ];
            let surface = (lib.eglCreatePbufferSurface)(display, config, surface_attrs.as_ptr());
            if surface == EGL_NO_SURFACE {
                (lib.eglTerminate)(display);
                return None;
            }
            let ctx_attrs = [EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE];
            let context =
                (lib.eglCreateContext)(display, config, EGL_NO_CONTEXT, ctx_attrs.as_ptr());
            if context == EGL_NO_CONTEXT {
                (lib.eglDestroySurface)(display, surface);
                (lib.eglTerminate)(display);
                return None;
            }
            if (lib.eglMakeCurrent)(display, surface, surface, context) == 0 {
                (lib.eglDestroyContext)(display, context);
                (lib.eglDestroySurface)(display, surface);
                (lib.eglTerminate)(display);
                return None;
            }
            Some(Self {
                display,
                surface,
                context,
                width,
                height,
            })
        }
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn should_close(&self) -> bool {
        false
    }

    fn pump_events(&mut self) -> Vec<BackendEvent> {
        Vec::new()
    }

    fn make_current(&self) -> bool {
        let Some(lib) = egl() else {
            return false;
        };
        unsafe { (lib.eglMakeCurrent)(self.display, self.surface, self.surface, self.context) != 0 }
    }

    fn swap_buffers(&self) {
        if let Some(lib) = egl() {
            unsafe {
                (lib.eglSwapBuffers)(self.display, self.surface);
            }
        }
    }

    fn get_proc_address(&self, name: &[u8]) -> *mut c_void {
        debug_assert_eq!(name.last(), Some(&0));
        let Some(lib) = egl() else {
            return core::ptr::null_mut();
        };
        let p = unsafe { (lib.eglGetProcAddress)(name.as_ptr() as *const c_char) };
        if !p.is_null() {
            return p;
        }
        if !lib.gles_handle.is_null() {
            return unsafe { dlsym(lib.gles_handle, name.as_ptr() as *const c_char) };
        }
        core::ptr::null_mut()
    }
}

impl Drop for AndroidWindow {
    fn drop(&mut self) {
        let Some(lib) = egl() else {
            return;
        };
        unsafe {
            (lib.eglMakeCurrent)(self.display, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
            if !self.context.is_null() {
                (lib.eglDestroyContext)(self.display, self.context);
            }
            if !self.surface.is_null() {
                (lib.eglDestroySurface)(self.display, self.surface);
            }
            (lib.eglTerminate)(self.display);
        }
    }
}

//! Hand-written FFI types and constants for the GL 4.3 / GLES 3.1 compute
//! pipeline used by [`super::gpu_raytracer::GpuRaytracer`].
//!
//! No external dependencies. Function pointers are resolved through the
//! existing offscreen / native window contexts in `crate::api::display`.

use core::ffi::{c_char, c_uint, c_void};

pub const GL_VENDOR: c_uint = 0x1F00;
pub const GL_RENDERER: c_uint = 0x1F01;
pub const GL_VERSION: c_uint = 0x1F02;

pub const GL_COMPUTE_SHADER: c_uint = 0x91B9;
pub const GL_SHADER_STORAGE_BUFFER: c_uint = 0x90D2;
pub const GL_DYNAMIC_DRAW: c_uint = 0x88E8;
pub const GL_STATIC_DRAW: c_uint = 0x88E4;
pub const GL_SHADER_STORAGE_BARRIER_BIT: c_uint = 0x0000_2000;
pub const GL_COMPILE_STATUS: c_uint = 0x8B81;
pub const GL_LINK_STATUS: c_uint = 0x8B82;
pub const GL_INFO_LOG_LENGTH: c_uint = 0x8B84;
pub const GL_MAP_READ_BIT: c_uint = 0x0001;

pub type GlGetString = unsafe extern "C" fn(c_uint) -> *const c_char;
pub type GlFinish = unsafe extern "C" fn();
pub type GlCreateShader = unsafe extern "C" fn(c_uint) -> c_uint;
pub type GlShaderSource = unsafe extern "C" fn(c_uint, i32, *const *const c_char, *const i32);
pub type GlCompileShader = unsafe extern "C" fn(c_uint);
pub type GlGetShaderiv = unsafe extern "C" fn(c_uint, c_uint, *mut i32);
pub type GlGetShaderInfoLog = unsafe extern "C" fn(c_uint, i32, *mut i32, *mut c_char);
pub type GlCreateProgram = unsafe extern "C" fn() -> c_uint;
pub type GlAttachShader = unsafe extern "C" fn(c_uint, c_uint);
pub type GlLinkProgram = unsafe extern "C" fn(c_uint);
pub type GlGetProgramiv = unsafe extern "C" fn(c_uint, c_uint, *mut i32);
pub type GlGetProgramInfoLog = unsafe extern "C" fn(c_uint, i32, *mut i32, *mut c_char);
pub type GlUseProgram = unsafe extern "C" fn(c_uint);
pub type GlGenBuffers = unsafe extern "C" fn(i32, *mut c_uint);
pub type GlBindBuffer = unsafe extern "C" fn(c_uint, c_uint);
pub type GlBufferData = unsafe extern "C" fn(c_uint, isize, *const c_void, c_uint);
pub type GlBindBufferBase = unsafe extern "C" fn(c_uint, c_uint, c_uint);
pub type GlDispatchCompute = unsafe extern "C" fn(c_uint, c_uint, c_uint);
pub type GlMemoryBarrier = unsafe extern "C" fn(c_uint);
pub type GlDeleteBuffers = unsafe extern "C" fn(i32, *const c_uint);
pub type GlDeleteProgram = unsafe extern "C" fn(c_uint);
pub type GlDeleteShader = unsafe extern "C" fn(c_uint);
pub type GlGetBufferSubData = unsafe extern "C" fn(c_uint, isize, isize, *mut c_void);
pub type GlMapBufferRange = unsafe extern "C" fn(c_uint, isize, isize, c_uint) -> *mut c_void;
pub type GlUnmapBuffer = unsafe extern "C" fn(c_uint) -> u8;

/// Bag of resolved GL function pointers used by the GPU ray-tracer.
#[derive(Clone, Copy)]
pub struct GlFns {
    /// `glGetString`.
    pub get_string: GlGetString,
    /// `glFinish`.
    pub finish: GlFinish,
    /// `glCreateShader`.
    pub create_shader: GlCreateShader,
    /// `glShaderSource`.
    pub shader_source: GlShaderSource,
    /// `glCompileShader`.
    pub compile_shader: GlCompileShader,
    /// `glGetShaderiv`.
    pub get_shader_iv: GlGetShaderiv,
    /// `glGetShaderInfoLog`.
    pub get_shader_info_log: GlGetShaderInfoLog,
    /// `glCreateProgram`.
    pub create_program: GlCreateProgram,
    /// `glAttachShader`.
    pub attach_shader: GlAttachShader,
    /// `glLinkProgram`.
    pub link_program: GlLinkProgram,
    /// `glGetProgramiv`.
    pub get_program_iv: GlGetProgramiv,
    /// `glGetProgramInfoLog`.
    pub get_program_info_log: GlGetProgramInfoLog,
    /// `glUseProgram`.
    pub use_program: GlUseProgram,
    /// `glGenBuffers`.
    pub gen_buffers: GlGenBuffers,
    /// `glBindBuffer`.
    pub bind_buffer: GlBindBuffer,
    /// `glBufferData`.
    pub buffer_data: GlBufferData,
    /// `glBindBufferBase`.
    pub bind_buffer_base: GlBindBufferBase,
    /// `glDispatchCompute`.
    pub dispatch_compute: GlDispatchCompute,
    /// `glMemoryBarrier`.
    pub memory_barrier: GlMemoryBarrier,
    /// `glDeleteBuffers`.
    pub delete_buffers: GlDeleteBuffers,
    /// `glDeleteProgram`.
    pub delete_program: GlDeleteProgram,
    /// `glDeleteShader`.
    pub delete_shader: GlDeleteShader,
    /// `glGetBufferSubData` — desktop GL only.
    pub get_buffer_sub_data: Option<GlGetBufferSubData>,
    /// `glMapBufferRange` — primary readback path on GLES.
    pub map_buffer_range: Option<GlMapBufferRange>,
    /// `glUnmapBuffer` — paired with `map_buffer_range`.
    pub unmap_buffer: Option<GlUnmapBuffer>,
}

/// Reads a NUL-terminated C string returned by `glGetString`.
pub fn read_cstring(p: *const c_char) -> String {
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

/// Reinterprets a raw `glXGetProcAddress` / `eglGetProcAddress` pointer as a
/// strongly-typed function pointer. Returns `None` for null pointers.
///
/// # Safety
/// The caller guarantees that the GL symbol referenced by `p` matches the
/// signature of `T`. Mismatches are undefined behaviour.
#[inline]
pub unsafe fn transmute_proc<T: Copy>(p: *mut c_void) -> Option<T> {
    if p.is_null() {
        None
    } else {
        Some(unsafe { core::mem::transmute_copy(&p) })
    }
}

/// Closure used to resolve a GL symbol from the active context.
pub type ProcLoader<'a> = &'a dyn Fn(&[u8]) -> *mut c_void;

macro_rules! load_required {
    ($loader:expr, $name:expr) => {{
        let p = ($loader)($name);
        if p.is_null() {
            return Err(format!(
                "GL symbol {:?} not found",
                core::str::from_utf8($name).unwrap_or("?")
            ));
        }
        unsafe { core::mem::transmute_copy::<*mut c_void, _>(&p) }
    }};
}

macro_rules! load_optional {
    ($loader:expr, $name:expr) => {{
        let p = ($loader)($name);
        unsafe { transmute_proc(p) }
    }};
}

impl GlFns {
    /// Resolves every required symbol through `loader`. Returns `Err` with a
    /// human-readable description on the first missing required function.
    pub fn load(loader: ProcLoader<'_>) -> Result<Self, String> {
        Ok(Self {
            get_string: load_required!(loader, b"glGetString\0"),
            finish: load_required!(loader, b"glFinish\0"),
            create_shader: load_required!(loader, b"glCreateShader\0"),
            shader_source: load_required!(loader, b"glShaderSource\0"),
            compile_shader: load_required!(loader, b"glCompileShader\0"),
            get_shader_iv: load_required!(loader, b"glGetShaderiv\0"),
            get_shader_info_log: load_required!(loader, b"glGetShaderInfoLog\0"),
            create_program: load_required!(loader, b"glCreateProgram\0"),
            attach_shader: load_required!(loader, b"glAttachShader\0"),
            link_program: load_required!(loader, b"glLinkProgram\0"),
            get_program_iv: load_required!(loader, b"glGetProgramiv\0"),
            get_program_info_log: load_required!(loader, b"glGetProgramInfoLog\0"),
            use_program: load_required!(loader, b"glUseProgram\0"),
            gen_buffers: load_required!(loader, b"glGenBuffers\0"),
            bind_buffer: load_required!(loader, b"glBindBuffer\0"),
            buffer_data: load_required!(loader, b"glBufferData\0"),
            bind_buffer_base: load_required!(loader, b"glBindBufferBase\0"),
            dispatch_compute: load_required!(loader, b"glDispatchCompute\0"),
            memory_barrier: load_required!(loader, b"glMemoryBarrier\0"),
            delete_buffers: load_required!(loader, b"glDeleteBuffers\0"),
            delete_program: load_required!(loader, b"glDeleteProgram\0"),
            delete_shader: load_required!(loader, b"glDeleteShader\0"),
            get_buffer_sub_data: load_optional!(loader, b"glGetBufferSubData\0"),
            map_buffer_range: load_optional!(loader, b"glMapBufferRange\0"),
            unmap_buffer: load_optional!(loader, b"glUnmapBuffer\0"),
        })
    }

    /// Reads `byte_len` bytes from the SSBO currently bound to
    /// `GL_SHADER_STORAGE_BUFFER` into `dst`, using whichever readback path
    /// the platform exposes (`glGetBufferSubData` on desktop,
    /// `glMapBufferRange` + `glUnmapBuffer` on GLES).
    ///
    /// Returns `Err` when no readback path is available.
    ///
    /// # Safety
    /// The caller guarantees that an SSBO of at least `byte_len` bytes is
    /// bound and that `dst` has matching size and alignment.
    pub unsafe fn read_ssbo(&self, dst: &mut [u8]) -> Result<(), &'static str> {
        let byte_len = dst.len() as isize;
        if let Some(get_sub) = self.get_buffer_sub_data {
            unsafe {
                get_sub(
                    GL_SHADER_STORAGE_BUFFER,
                    0,
                    byte_len,
                    dst.as_mut_ptr() as *mut c_void,
                );
            }
            return Ok(());
        }
        if let (Some(map), Some(unmap)) = (self.map_buffer_range, self.unmap_buffer) {
            let ptr = unsafe { map(GL_SHADER_STORAGE_BUFFER, 0, byte_len, GL_MAP_READ_BIT) };
            if ptr.is_null() {
                return Err("glMapBufferRange returned NULL");
            }
            unsafe {
                core::ptr::copy_nonoverlapping(ptr as *const u8, dst.as_mut_ptr(), dst.len());
                unmap(GL_SHADER_STORAGE_BUFFER);
            }
            return Ok(());
        }
        Err("no SSBO readback function available")
    }

    /// Compiles a single compute shader, returning its GL name on success or
    /// the driver info log on failure.
    ///
    /// # Safety
    /// A current GL context capable of compute shaders must be bound to the
    /// calling thread.
    pub unsafe fn compile_compute(&self, source: &str) -> Result<c_uint, String> {
        let shader = unsafe { (self.create_shader)(GL_COMPUTE_SHADER) };
        if shader == 0 {
            return Err("glCreateShader returned 0".into());
        }
        let src_ptr: *const c_char = source.as_ptr() as *const c_char;
        let src_len: i32 = source.len() as i32;
        unsafe {
            (self.shader_source)(shader, 1, &src_ptr, &src_len);
            (self.compile_shader)(shader);
        }
        let mut status: i32 = 0;
        unsafe { (self.get_shader_iv)(shader, GL_COMPILE_STATUS, &mut status) };
        if status == 0 {
            let log = unsafe { self.fetch_shader_info_log(shader) };
            unsafe { (self.delete_shader)(shader) };
            return Err(format!("compute shader compile failed: {log}"));
        }
        Ok(shader)
    }

    /// Links a fresh program containing the given compute shader. Returns the
    /// program GL name on success or the driver info log on failure.
    ///
    /// # Safety
    /// `shader` must be a valid compiled compute shader. A current GL context
    /// must be bound.
    pub unsafe fn link_program(&self, shader: c_uint) -> Result<c_uint, String> {
        let program = unsafe { (self.create_program)() };
        if program == 0 {
            return Err("glCreateProgram returned 0".into());
        }
        unsafe {
            (self.attach_shader)(program, shader);
            (self.link_program)(program);
        }
        let mut status: i32 = 0;
        unsafe { (self.get_program_iv)(program, GL_LINK_STATUS, &mut status) };
        if status == 0 {
            let log = unsafe { self.fetch_program_info_log(program) };
            unsafe { (self.delete_program)(program) };
            return Err(format!("program link failed: {log}"));
        }
        Ok(program)
    }

    unsafe fn fetch_shader_info_log(&self, shader: c_uint) -> String {
        let mut len: i32 = 0;
        unsafe { (self.get_shader_iv)(shader, GL_INFO_LOG_LENGTH, &mut len) };
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u8; len as usize];
        let mut written: i32 = 0;
        unsafe {
            (self.get_shader_info_log)(shader, len, &mut written, buf.as_mut_ptr() as *mut c_char);
        }
        String::from_utf8_lossy(&buf[..written.max(0) as usize]).into_owned()
    }

    unsafe fn fetch_program_info_log(&self, program: c_uint) -> String {
        let mut len: i32 = 0;
        unsafe { (self.get_program_iv)(program, GL_INFO_LOG_LENGTH, &mut len) };
        if len <= 0 {
            return String::new();
        }
        let mut buf = vec![0u8; len as usize];
        let mut written: i32 = 0;
        unsafe {
            (self.get_program_info_log)(
                program,
                len,
                &mut written,
                buf.as_mut_ptr() as *mut c_char,
            );
        }
        String::from_utf8_lossy(&buf[..written.max(0) as usize]).into_owned()
    }
}

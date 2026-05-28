use core::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

use enginerenderer::api::display::NativeWindow;

pub type GLenum = c_uint;
pub type GLboolean = c_uchar;
pub type GLbitfield = c_uint;
pub type GLint = c_int;
pub type GLuint = c_uint;
pub type GLsizei = c_int;
pub type GLfloat = f32;
pub type GLchar = c_char;
pub type GLsizeiptr = isize;
pub type GLintptr = isize;

pub const GL_COLOR_BUFFER_BIT: GLbitfield = 0x0000_4000;
pub const GL_DEPTH_BUFFER_BIT: GLbitfield = 0x0000_0100;
pub const GL_TRIANGLES: GLenum = 0x0004;
pub const GL_LINES: GLenum = 0x0001;
pub const GL_FLOAT: GLenum = 0x1406;
pub const GL_FALSE: GLboolean = 0;
pub const GL_TRUE: GLboolean = 1;
pub const GL_ARRAY_BUFFER: GLenum = 0x8892;
pub const GL_DYNAMIC_DRAW: GLenum = 0x88E8;
pub const GL_VERTEX_SHADER: GLenum = 0x8B31;
pub const GL_FRAGMENT_SHADER: GLenum = 0x8B30;
pub const GL_COMPILE_STATUS: GLenum = 0x8B81;
pub const GL_LINK_STATUS: GLenum = 0x8B82;
pub const GL_TEXTURE_2D: GLenum = 0x0DE1;
pub const GL_TEXTURE0: GLenum = 0x84C0;
pub const GL_TEXTURE_MIN_FILTER: GLenum = 0x2801;
pub const GL_TEXTURE_MAG_FILTER: GLenum = 0x2800;
pub const GL_TEXTURE_WRAP_S: GLenum = 0x2802;
pub const GL_TEXTURE_WRAP_T: GLenum = 0x2803;
pub const GL_NEAREST: GLint = 0x2600;
pub const GL_LINEAR: GLint = 0x2601;
pub const GL_CLAMP_TO_EDGE: GLint = 0x812F;
pub const GL_R8: GLint = 0x8229;
pub const GL_RED: GLenum = 0x1903;
pub const GL_RGBA: GLenum = 0x1908;
pub const GL_RGBA8: GLint = 0x8058;
pub const GL_UNSIGNED_BYTE: GLenum = 0x1401;
pub const GL_UNPACK_ALIGNMENT: GLenum = 0x0CF5;
pub const GL_BLEND: GLenum = 0x0BE2;
pub const GL_SRC_ALPHA: GLenum = 0x0302;
pub const GL_ONE_MINUS_SRC_ALPHA: GLenum = 0x0303;
pub const GL_SCISSOR_TEST: GLenum = 0x0C11;

#[derive(Default)]
pub struct GlFns {
    pub clear_color: Option<extern "system" fn(GLfloat, GLfloat, GLfloat, GLfloat)>,
    pub clear: Option<extern "system" fn(GLbitfield)>,
    pub viewport: Option<extern "system" fn(GLint, GLint, GLsizei, GLsizei)>,
    pub enable: Option<extern "system" fn(GLenum)>,
    pub disable: Option<extern "system" fn(GLenum)>,
    pub blend_func: Option<extern "system" fn(GLenum, GLenum)>,
    pub scissor: Option<extern "system" fn(GLint, GLint, GLsizei, GLsizei)>,
    pub gen_buffers: Option<extern "system" fn(GLsizei, *mut GLuint)>,
    pub bind_buffer: Option<extern "system" fn(GLenum, GLuint)>,
    pub buffer_data: Option<extern "system" fn(GLenum, GLsizeiptr, *const c_void, GLenum)>,
    pub buffer_sub_data: Option<extern "system" fn(GLenum, GLintptr, GLsizeiptr, *const c_void)>,
    pub gen_vertex_arrays: Option<extern "system" fn(GLsizei, *mut GLuint)>,
    pub bind_vertex_array: Option<extern "system" fn(GLuint)>,
    pub enable_vertex_attrib_array: Option<extern "system" fn(GLuint)>,
    pub vertex_attrib_pointer: Option<
        extern "system" fn(GLuint, GLint, GLenum, GLboolean, GLsizei, *const c_void),
    >,
    pub create_shader: Option<extern "system" fn(GLenum) -> GLuint>,
    pub shader_source:
        Option<extern "system" fn(GLuint, GLsizei, *const *const GLchar, *const GLint)>,
    pub compile_shader: Option<extern "system" fn(GLuint)>,
    pub get_shader_iv: Option<extern "system" fn(GLuint, GLenum, *mut GLint)>,
    pub create_program: Option<extern "system" fn() -> GLuint>,
    pub attach_shader: Option<extern "system" fn(GLuint, GLuint)>,
    pub link_program: Option<extern "system" fn(GLuint)>,
    pub get_program_iv: Option<extern "system" fn(GLuint, GLenum, *mut GLint)>,
    pub use_program: Option<extern "system" fn(GLuint)>,
    pub get_uniform_location: Option<extern "system" fn(GLuint, *const GLchar) -> GLint>,
    pub uniform2f: Option<extern "system" fn(GLint, GLfloat, GLfloat)>,
    pub uniform1i: Option<extern "system" fn(GLint, GLint)>,
    pub draw_arrays: Option<extern "system" fn(GLenum, GLint, GLsizei)>,
    pub gen_textures: Option<extern "system" fn(GLsizei, *mut GLuint)>,
    pub delete_textures: Option<extern "system" fn(GLsizei, *const GLuint)>,
    pub bind_texture: Option<extern "system" fn(GLenum, GLuint)>,
    pub tex_image_2d: Option<
        extern "system" fn(
            GLenum,
            GLint,
            GLint,
            GLsizei,
            GLsizei,
            GLint,
            GLenum,
            GLenum,
            *const c_void,
        ),
    >,
    pub tex_parameter_i: Option<extern "system" fn(GLenum, GLenum, GLint)>,
    pub active_texture: Option<extern "system" fn(GLenum)>,
    pub pixel_storei: Option<extern "system" fn(GLenum, GLint)>,
}

unsafe fn nul(name: &[u8]) -> [u8; 64] {
    let mut out = [0u8; 64];
    let n = name.len().min(63);
    out[..n].copy_from_slice(&name[..n]);
    out[n] = 0;
    out
}

fn load<T>(window: &NativeWindow, name: &[u8]) -> Option<T> {
    let buf = unsafe { nul(name) };
    let ptr = window.gl_get_proc(&buf[..=name.len().min(63)]);
    if ptr.is_null() {
        return None;
    }
    Some(unsafe { core::mem::transmute_copy::<*mut c_void, T>(&ptr) })
}

impl GlFns {
    pub fn load(window: &NativeWindow) -> Self {
        Self {
            clear_color: load(window, b"glClearColor"),
            clear: load(window, b"glClear"),
            viewport: load(window, b"glViewport"),
            enable: load(window, b"glEnable"),
            disable: load(window, b"glDisable"),
            blend_func: load(window, b"glBlendFunc"),
            scissor: load(window, b"glScissor"),
            gen_buffers: load(window, b"glGenBuffers"),
            bind_buffer: load(window, b"glBindBuffer"),
            buffer_data: load(window, b"glBufferData"),
            buffer_sub_data: load(window, b"glBufferSubData"),
            gen_vertex_arrays: load(window, b"glGenVertexArrays"),
            bind_vertex_array: load(window, b"glBindVertexArray"),
            enable_vertex_attrib_array: load(window, b"glEnableVertexAttribArray"),
            vertex_attrib_pointer: load(window, b"glVertexAttribPointer"),
            create_shader: load(window, b"glCreateShader"),
            shader_source: load(window, b"glShaderSource"),
            compile_shader: load(window, b"glCompileShader"),
            get_shader_iv: load(window, b"glGetShaderiv"),
            create_program: load(window, b"glCreateProgram"),
            attach_shader: load(window, b"glAttachShader"),
            link_program: load(window, b"glLinkProgram"),
            get_program_iv: load(window, b"glGetProgramiv"),
            use_program: load(window, b"glUseProgram"),
            get_uniform_location: load(window, b"glGetUniformLocation"),
            uniform2f: load(window, b"glUniform2f"),
            uniform1i: load(window, b"glUniform1i"),
            draw_arrays: load(window, b"glDrawArrays"),
            gen_textures: load(window, b"glGenTextures"),
            delete_textures: load(window, b"glDeleteTextures"),
            bind_texture: load(window, b"glBindTexture"),
            tex_image_2d: load(window, b"glTexImage2D"),
            tex_parameter_i: load(window, b"glTexParameteri"),
            active_texture: load(window, b"glActiveTexture"),
            pixel_storei: load(window, b"glPixelStorei"),
        }
    }

    pub fn complete(&self) -> bool {
        self.clear_color.is_some()
            && self.clear.is_some()
            && self.viewport.is_some()
            && self.gen_buffers.is_some()
            && self.bind_buffer.is_some()
            && self.buffer_data.is_some()
            && self.gen_vertex_arrays.is_some()
            && self.bind_vertex_array.is_some()
            && self.enable_vertex_attrib_array.is_some()
            && self.vertex_attrib_pointer.is_some()
            && self.create_shader.is_some()
            && self.shader_source.is_some()
            && self.compile_shader.is_some()
            && self.create_program.is_some()
            && self.attach_shader.is_some()
            && self.link_program.is_some()
            && self.use_program.is_some()
            && self.get_uniform_location.is_some()
            && self.uniform2f.is_some()
            && self.draw_arrays.is_some()
            && self.gen_textures.is_some()
            && self.bind_texture.is_some()
            && self.tex_image_2d.is_some()
            && self.tex_parameter_i.is_some()
            && self.active_texture.is_some()
    }
}

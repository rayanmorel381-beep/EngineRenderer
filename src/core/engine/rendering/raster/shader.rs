use std::collections::HashMap;
use std::ffi::CString;

pub struct ShaderProgram {
    handle: u32,
    vertex_src: String,
    fragment_src: String,
    uniforms: HashMap<String, i32>,
}

pub struct ShaderCache {
    programs: HashMap<String, ShaderProgram>,
}

impl ShaderProgram {
    pub fn from_sources(vertex_src: &str, fragment_src: &str) -> Result<Self, String> {
        let program = ShaderProgram {
            handle: 0,
            vertex_src: vertex_src.to_string(),
            fragment_src: fragment_src.to_string(),
            uniforms: HashMap::new(),
        };
        Ok(program)
    }

    pub fn compile(&mut self, gl_procs: &GlProcs) -> Result<(), String> {
        unsafe {
            let vs = self.compile_shader(
                &self.vertex_src,
                0x8B31,
                gl_procs,
            )?;
            let fs = self.compile_shader(
                &self.fragment_src,
                0x8B30,
                gl_procs,
            )?;

            let program = (gl_procs.glCreateProgram)();
            (gl_procs.glAttachShader)(program, vs);
            (gl_procs.glAttachShader)(program, fs);
            (gl_procs.glLinkProgram)(program);

            let mut linked = 0i32;
            (gl_procs.glGetProgramiv)(program, 0x8B82, &mut linked);
            if linked == 0 {
                return Err("Program linking failed".to_string());
            }

            (gl_procs.glDeleteShader)(vs);
            (gl_procs.glDeleteShader)(fs);

            self.handle = program;
            Ok(())
        }
    }

    unsafe fn compile_shader(
        &self,
        src: &str,
        shader_type: u32,
        gl_procs: &GlProcs,
    ) -> Result<u32, String> {
        let shader = (gl_procs.glCreateShader)(shader_type);
        let c_src = CString::new(src).map_err(|_| "Shader source contains interior NUL byte".to_string())?;
        let src_ptr = c_src.as_ptr();
        (gl_procs.glShaderSource)(shader, 1, &src_ptr, std::ptr::null());
        (gl_procs.glCompileShader)(shader);

        let mut compiled = 0i32;
        (gl_procs.glGetShaderiv)(shader, 0x8B81, &mut compiled);
        if compiled == 0 {
            return Err("Shader compilation failed".to_string());
        }
        Ok(shader)
    }

    pub fn set_uniform_1f(&self, name: &str, value: f32, gl_procs: &GlProcs) {
        unsafe {
            let Ok(loc) = self.get_uniform_location(name, gl_procs) else {
                return;
            };
            (gl_procs.glUniform1f)(loc, value);
        }
    }

    pub fn set_uniform_3f(&self, name: &str, x: f32, y: f32, z: f32, gl_procs: &GlProcs) {
        unsafe {
            let Ok(loc) = self.get_uniform_location(name, gl_procs) else {
                return;
            };
            (gl_procs.glUniform3f)(loc, x, y, z);
        }
    }

    pub fn set_uniform_4f(&self, name: &str, x: f32, y: f32, z: f32, w: f32, gl_procs: &GlProcs) {
        unsafe {
            let Ok(loc) = self.get_uniform_location(name, gl_procs) else {
                return;
            };
            (gl_procs.glUniform4f)(loc, x, y, z, w);
        }
    }

    pub fn set_uniform_matrix4f(&self, name: &str, matrix: &[f32; 16], gl_procs: &GlProcs) {
        unsafe {
            let Ok(loc) = self.get_uniform_location(name, gl_procs) else {
                return;
            };
            (gl_procs.glUniformMatrix4fv)(loc, 1, 0, matrix.as_ptr());
        }
    }

    unsafe fn get_uniform_location(&self, name: &str, gl_procs: &GlProcs) -> Result<i32, String> {
        let c_name = CString::new(name).map_err(|_| "Uniform name contains interior NUL byte".to_string())?;
        Ok((gl_procs.glGetUniformLocation)(self.handle, c_name.as_ptr()))
    }

    pub fn use_program(&self, gl_procs: &GlProcs) {
        unsafe {
            (gl_procs.glUseProgram)(self.handle);
        }
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }
}

pub struct GlProcs {
    pub glCreateShader: unsafe extern "C" fn(u32) -> u32,
    pub glShaderSource: unsafe extern "C" fn(u32, i32, *const *const i8, *const i32),
    pub glCompileShader: unsafe extern "C" fn(u32),
    pub glGetShaderiv: unsafe extern "C" fn(u32, u32, *mut i32),
    pub glCreateProgram: unsafe extern "C" fn() -> u32,
    pub glAttachShader: unsafe extern "C" fn(u32, u32),
    pub glLinkProgram: unsafe extern "C" fn(u32),
    pub glGetProgramiv: unsafe extern "C" fn(u32, u32, *mut i32),
    pub glDeleteShader: unsafe extern "C" fn(u32),
    pub glUseProgram: unsafe extern "C" fn(u32),
    pub glGetUniformLocation: unsafe extern "C" fn(u32, *const i8) -> i32,
    pub glUniform1f: unsafe extern "C" fn(i32, f32),
    pub glUniform3f: unsafe extern "C" fn(i32, f32, f32, f32),
    pub glUniform4f: unsafe extern "C" fn(i32, f32, f32, f32, f32),
    pub glUniformMatrix4fv: unsafe extern "C" fn(i32, i32, u8, *const f32),
}

impl ShaderCache {
    pub fn new() -> Self {
        ShaderCache {
            programs: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, key: &str, vertex: &str, fragment: &str) -> Result<&ShaderProgram, String> {
        if !self.programs.contains_key(key) {
            let program = ShaderProgram::from_sources(vertex, fragment)?;
            self.programs.insert(key.to_string(), program);
        }
        self.programs
            .get(key)
            .ok_or_else(|| format!("Missing shader program for key '{}'", key))
    }
}

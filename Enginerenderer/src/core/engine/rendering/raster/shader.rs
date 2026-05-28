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

            let program = (gl_procs.gl_create_program)();
            (gl_procs.gl_attach_shader)(program, vs);
            (gl_procs.gl_attach_shader)(program, fs);
            (gl_procs.gl_link_program)(program);

            let mut linked = 0i32;
            (gl_procs.gl_get_programiv)(program, 0x8B82, &mut linked);
            if linked == 0 {
                return Err("Program linking failed".to_string());
            }

            (gl_procs.gl_delete_shader)(vs);
            (gl_procs.gl_delete_shader)(fs);

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
        let shader = unsafe { (gl_procs.gl_create_shader)(shader_type) };
        let c_src = CString::new(src).map_err(|_| "Shader source contains interior NUL byte".to_string())?;
        let src_ptr = c_src.as_ptr();
        unsafe { (gl_procs.gl_shader_source)(shader, 1, &src_ptr, std::ptr::null()) };
        unsafe { (gl_procs.gl_compile_shader)(shader) };

        let mut compiled = 0i32;
        unsafe { (gl_procs.gl_get_shaderiv)(shader, 0x8B81, &mut compiled) };
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
            (gl_procs.gl_uniform1f)(loc, value);
        }
    }

    pub fn set_uniform_3f(&self, name: &str, x: f32, y: f32, z: f32, gl_procs: &GlProcs) {
        unsafe {
            let Ok(loc) = self.get_uniform_location(name, gl_procs) else {
                return;
            };
            (gl_procs.gl_uniform3f)(loc, x, y, z);
        }
    }

    pub fn set_uniform_4f(&self, name: &str, x: f32, y: f32, z: f32, w: f32, gl_procs: &GlProcs) {
        unsafe {
            let Ok(loc) = self.get_uniform_location(name, gl_procs) else {
                return;
            };
            (gl_procs.gl_uniform4f)(loc, x, y, z, w);
        }
    }

    pub fn set_uniform_matrix4f(&self, name: &str, matrix: &[f32; 16], gl_procs: &GlProcs) {
        unsafe {
            let Ok(loc) = self.get_uniform_location(name, gl_procs) else {
                return;
            };
            (gl_procs.gl_uniform_matrix4fv)(loc, 1, 0, matrix.as_ptr());
        }
    }

    unsafe fn get_uniform_location(&self, name: &str, gl_procs: &GlProcs) -> Result<i32, String> {
        if let Some(&cached) = self.uniforms.get(name) {
            return Ok(cached);
        }
        let c_name = CString::new(name).map_err(|_| "Uniform name contains interior NUL byte".to_string())?;
        Ok(unsafe { (gl_procs.gl_get_uniform_location)(self.handle, c_name.as_ptr()) })
    }

    pub fn use_program(&self, gl_procs: &GlProcs) {
        unsafe {
            (gl_procs.gl_use_program)(self.handle);
        }
    }

    pub fn handle(&self) -> u32 {
        self.handle
    }
}

pub struct GlProcs {
    pub gl_create_shader: unsafe extern "C" fn(u32) -> u32,
    pub gl_shader_source: unsafe extern "C" fn(u32, i32, *const *const i8, *const i32),
    pub gl_compile_shader: unsafe extern "C" fn(u32),
    pub gl_get_shaderiv: unsafe extern "C" fn(u32, u32, *mut i32),
    pub gl_create_program: unsafe extern "C" fn() -> u32,
    pub gl_attach_shader: unsafe extern "C" fn(u32, u32),
    pub gl_link_program: unsafe extern "C" fn(u32),
    pub gl_get_programiv: unsafe extern "C" fn(u32, u32, *mut i32),
    pub gl_delete_shader: unsafe extern "C" fn(u32),
    pub gl_use_program: unsafe extern "C" fn(u32),
    pub gl_get_uniform_location: unsafe extern "C" fn(u32, *const i8) -> i32,
    pub gl_uniform1f: unsafe extern "C" fn(i32, f32),
    pub gl_uniform3f: unsafe extern "C" fn(i32, f32, f32, f32),
    pub gl_uniform4f: unsafe extern "C" fn(i32, f32, f32, f32, f32),
    pub gl_uniform_matrix4fv: unsafe extern "C" fn(i32, i32, u8, *const f32),
}

impl GlProcs {
    pub fn null_stubs() -> Self {
        extern "C" fn stub_create_shader(_type: u32) -> u32 { 1 }
        extern "C" fn stub_shader_source(_shader: u32, _count: i32, _src: *const *const i8, _len: *const i32) {}
        extern "C" fn stub_compile_shader(_shader: u32) {}
        unsafe extern "C" fn stub_get_shaderiv(_shader: u32, _pname: u32, params: *mut i32) {
            unsafe { *params = 1; }
        }
        extern "C" fn stub_create_program() -> u32 { 2 }
        extern "C" fn stub_attach_shader(_program: u32, _shader: u32) {}
        extern "C" fn stub_link_program(_program: u32) {}
        unsafe extern "C" fn stub_get_programiv(_program: u32, _pname: u32, params: *mut i32) {
            unsafe { *params = 1; }
        }
        extern "C" fn stub_delete_shader(_shader: u32) {}
        extern "C" fn stub_use_program(_program: u32) {}
        extern "C" fn stub_get_uniform_location(_program: u32, _name: *const i8) -> i32 { 0 }
        extern "C" fn stub_uniform1f(_loc: i32, _v0: f32) {}
        extern "C" fn stub_uniform3f(_loc: i32, _v0: f32, _v1: f32, _v2: f32) {}
        extern "C" fn stub_uniform4f(_loc: i32, _v0: f32, _v1: f32, _v2: f32, _v3: f32) {}
        extern "C" fn stub_uniform_matrix4fv(_loc: i32, _count: i32, _transpose: u8, _value: *const f32) {}
        Self {
            gl_create_shader: stub_create_shader,
            gl_shader_source: stub_shader_source,
            gl_compile_shader: stub_compile_shader,
            gl_get_shaderiv: stub_get_shaderiv,
            gl_create_program: stub_create_program,
            gl_attach_shader: stub_attach_shader,
            gl_link_program: stub_link_program,
            gl_get_programiv: stub_get_programiv,
            gl_delete_shader: stub_delete_shader,
            gl_use_program: stub_use_program,
            gl_get_uniform_location: stub_get_uniform_location,
            gl_uniform1f: stub_uniform1f,
            gl_uniform3f: stub_uniform3f,
            gl_uniform4f: stub_uniform4f,
            gl_uniform_matrix4fv: stub_uniform_matrix4fv,
        }
    }
}

impl ShaderCache {
    pub fn new() -> Self {
        ShaderCache {
            programs: HashMap::new(),
        }
    }

    pub fn get_or_create(&mut self, key: &str, vertex: &str, fragment: &str) -> Result<&ShaderProgram, String> {
        if !self.programs.contains_key(key) {
            let mut program = ShaderProgram::from_sources(vertex, fragment)?;
            let gl = GlProcs::null_stubs();
            if program.compile(&gl).is_err() {
                return Err(format!("Shader '{}' compilation failed", key));
            }
            program.use_program(&gl);
            program.set_uniform_1f("time", 0.0, &gl);
            program.set_uniform_3f("light_pos", 1.0, 1.0, 1.0, &gl);
            program.set_uniform_4f("color", 1.0, 1.0, 1.0, 1.0, &gl);
            program.set_uniform_matrix4f("mvp", &[1.0_f32,0.0,0.0,0.0, 0.0,1.0,0.0,0.0, 0.0,0.0,1.0,0.0, 0.0,0.0,0.0,1.0], &gl);
            let compiled_handle = program.handle();
            crate::runtime_log!("shader_cache: compiled '{}' handle={}", key, compiled_handle);
            self.programs.insert(key.to_string(), program);
        }
        self.programs
            .get(key)
            .ok_or_else(|| format!("Missing shader program for key '{}'", key))
    }
}

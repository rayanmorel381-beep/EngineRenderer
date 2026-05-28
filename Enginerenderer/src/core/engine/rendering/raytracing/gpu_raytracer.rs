//! GPU compute path-tracer that renders an entire image in one
//! `glDispatchCompute`, then reads the result back into a [`FrameBuffer`].

use core::ffi::{c_uint, c_void};

use super::camera::Camera;
use super::gpu_gl::{
    read_cstring, GlFns, GL_DYNAMIC_DRAW, GL_RENDERER, GL_SHADER_STORAGE_BARRIER_BIT,
    GL_SHADER_STORAGE_BUFFER, GL_STATIC_DRAW, GL_VENDOR, GL_VERSION,
};
use super::gpu_bvh::{build as build_bvh, pack_nodes as pack_bvh_nodes, pack_prim_refs};
use super::gpu_scene_pack::{
    pack_area_lights, pack_frame, pack_spheres, pack_triangles, GpuFrameConfig,
};
use super::gpu_shader::{assemble, bindings};
use super::math::Vec3;
use super::scene::Scene;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

/// Per-render configuration for [`GpuRaytracer::render`].
#[derive(Debug, Clone, Copy)]
pub struct GpuRenderConfig {
    pub width: u32,
    pub height: u32,
    pub samples: u32,
    pub max_bounces: u32,
    pub seed: u32,
    /// Linear exposure multiplier applied before ACES tonemapping.
    pub exposure: f32,
    /// Optional 3×3 edge-aware denoise pass on the readback buffer.
    pub denoise: bool,
}

impl Default for GpuRenderConfig {
    fn default() -> Self {
        Self {
            width: 256,
            height: 256,
            samples: 4,
            max_bounces: 4,
            seed: 0xC0FFEE_u32,
            exposure: 1.0,
            denoise: false,
        }
    }
}

/// Identifying strings reported by the live GL driver.
#[derive(Debug, Clone)]
pub struct GpuDeviceInfo {
    pub vendor: String,
    pub renderer: String,
    pub version: String,
}

/// Resources owned by the GPU path tracer for the lifetime of the GL context.
pub struct GpuRaytracer {
    gl: GlFns,
    program: c_uint,
    shader: c_uint,
    output_ssbo: c_uint,
    frame_ssbo: c_uint,
    sphere_ssbo: c_uint,
    triangle_ssbo: c_uint,
    area_light_ssbo: c_uint,
    bvh_nodes_ssbo: c_uint,
    bvh_prims_ssbo: c_uint,
    output_capacity_bytes: usize,
    device: GpuDeviceInfo,
}

impl GpuRaytracer {
    /// Builds a path tracer on top of a current GL 4.3 / GLES 3.1 context.
    pub fn new(
        proc_loader: &dyn Fn(&[u8]) -> *mut c_void,
        is_es: bool,
    ) -> Result<Self, String> {
        let gl = GlFns::load(&proc_loader)?;
        let source = assemble(is_es);
        let shader = unsafe { gl.compile_compute(&source) }?;
        let program = match unsafe { gl.link_program(shader) } {
            Ok(p) => p,
            Err(e) => {
                unsafe { (gl.delete_shader)(shader) };
                return Err(e);
            }
        };

        let device = unsafe {
            GpuDeviceInfo {
                vendor: read_cstring((gl.get_string)(GL_VENDOR)),
                renderer: read_cstring((gl.get_string)(GL_RENDERER)),
                version: read_cstring((gl.get_string)(GL_VERSION)),
            }
        };

        let mut ssbos = [0u32; 7];
        unsafe { (gl.gen_buffers)(7, ssbos.as_mut_ptr()) };

        Ok(Self {
            gl,
            program,
            shader,
            output_ssbo: ssbos[0],
            frame_ssbo: ssbos[1],
            sphere_ssbo: ssbos[2],
            triangle_ssbo: ssbos[3],
            area_light_ssbo: ssbos[4],
            bvh_nodes_ssbo: ssbos[5],
            bvh_prims_ssbo: ssbos[6],
            output_capacity_bytes: 0,
            device,
        })
    }

    pub fn device(&self) -> &GpuDeviceInfo {
        &self.device
    }

    /// Renders `scene` from `camera` into a fresh [`FrameBuffer`].
    pub fn render(
        &mut self,
        scene: &Scene,
        camera: &Camera,
        cfg: GpuRenderConfig,
    ) -> Result<FrameBuffer, String> {
        if cfg.width == 0 || cfg.height == 0 {
            return Err("image dimensions must be non-zero".into());
        }

        let pixel_count = cfg.width as usize * cfg.height as usize;
        let prim_count = scene.objects.len() + scene.triangles.len();
        let area_count = scene.area_lights.len();
        let bvh_log = ((prim_count.max(2) as f64).log2().ceil() as usize).max(1);
        let shadow_tests_per_hit = 2 + area_count;
        let bounces = (cfg.max_bounces as usize) + 1;
        let workload = pixel_count
            .saturating_mul(cfg.samples.max(1) as usize)
            .saturating_mul(bounces)
            .saturating_mul(shadow_tests_per_hit)
            .saturating_mul(bvh_log);
        const MAX_WORKLOAD: u64 = 5_000_000_000;
        if workload as u64 > MAX_WORKLOAD {
            return Err(format!(
                "GPU workload guard: estimated {workload} ray-steps exceeds safety cap {MAX_WORKLOAD} (would risk GPU TDR / desktop freeze). Reduce width*height*samples*(max_bounces+1)*(2+area_lights)*log2(prims)."
            ));
        }

        let output_bytes = pixel_count * 4 * core::mem::size_of::<f32>();

        let frame_blob = pack_frame(
            camera,
            scene,
            GpuFrameConfig {
                width: cfg.width,
                height: cfg.height,
                samples: cfg.samples.max(1),
                max_bounces: cfg.max_bounces,
                seed: cfg.seed,
                exposure: cfg.exposure,
            },
        );
        let sphere_blob = pack_spheres(&scene.objects);
        let triangle_blob = pack_triangles(&scene.triangles);
        let area_blob = pack_area_lights(&scene.area_lights);
        let bvh = build_bvh(scene);
        let bvh_nodes_blob = pack_bvh_nodes(&bvh.nodes);
        let bvh_prims_blob = pack_prim_refs(&bvh.prim_refs);

        unsafe {
            self.upload_output(output_bytes);
            self.upload_static(self.frame_ssbo, &frame_blob);
            self.upload_static(self.sphere_ssbo, &sphere_blob);
            self.upload_static(self.triangle_ssbo, &triangle_blob);
            self.upload_static(self.area_light_ssbo, &area_blob);
            self.upload_static(self.bvh_nodes_ssbo, &bvh_nodes_blob);
            self.upload_static(self.bvh_prims_ssbo, &bvh_prims_blob);
            self.bind_all();
            (self.gl.use_program)(self.program);
            let groups_x = cfg.width.div_ceil(8);
            let groups_y = cfg.height.div_ceil(8);
            (self.gl.dispatch_compute)(groups_x, groups_y, 1);
            (self.gl.memory_barrier)(GL_SHADER_STORAGE_BARRIER_BIT);
            (self.gl.finish)();
        }

        let mut raw = vec![0u8; output_bytes];
        unsafe {
            (self.gl.bind_buffer)(GL_SHADER_STORAGE_BUFFER, self.output_ssbo);
            self.gl.read_ssbo(&mut raw)?;
        }

        let f32_count = pixel_count * 4;
        let floats: &[f32] = unsafe {
            core::slice::from_raw_parts(raw.as_ptr() as *const f32, f32_count)
        };

        let mut color: Vec<Vec3> = Vec::with_capacity(pixel_count);
        let mut alpha: Vec<f64> = Vec::with_capacity(pixel_count);
        for chunk in floats.chunks_exact(4) {
            color.push(Vec3::new(chunk[0] as f64, chunk[1] as f64, chunk[2] as f64));
            alpha.push(chunk[3] as f64);
        }

        if cfg.denoise {
            denoise_bilateral(&mut color, cfg.width as usize, cfg.height as usize);
        }

        let mut framebuffer = FrameBuffer::new(cfg.width as usize, cfg.height as usize);
        let w = cfg.width as usize;
        let h = cfg.height as usize;
        for y in 0..h {
            for x in 0..w {
                let src = y * w + x;
                let dst = (h - 1 - y) * w + x;
                framebuffer.color[dst] = color[src];
                framebuffer.alpha[dst] = alpha[src];
                framebuffer.sample_count[dst] = cfg.samples.max(1);
            }
        }
        Ok(framebuffer)
    }

    unsafe fn bind_all(&self) {
        unsafe {
            (self.gl.bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, bindings::OUTPUT, self.output_ssbo);
            (self.gl.bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, bindings::FRAME, self.frame_ssbo);
            (self.gl.bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, bindings::SPHERES, self.sphere_ssbo);
            (self.gl.bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, bindings::TRIANGLES, self.triangle_ssbo);
            (self.gl.bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, bindings::AREA_LIGHTS, self.area_light_ssbo);
            (self.gl.bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, bindings::BVH_NODES, self.bvh_nodes_ssbo);
            (self.gl.bind_buffer_base)(GL_SHADER_STORAGE_BUFFER, bindings::BVH_PRIMS, self.bvh_prims_ssbo);
        }
    }

    unsafe fn upload_output(&mut self, byte_len: usize) {
        unsafe {
            (self.gl.bind_buffer)(GL_SHADER_STORAGE_BUFFER, self.output_ssbo);
            if byte_len != self.output_capacity_bytes {
                (self.gl.buffer_data)(
                    GL_SHADER_STORAGE_BUFFER,
                    byte_len as isize,
                    core::ptr::null(),
                    GL_DYNAMIC_DRAW,
                );
                self.output_capacity_bytes = byte_len;
            }
        }
    }

    unsafe fn upload_static(&self, ssbo: c_uint, data: &[u8]) {
        unsafe {
            (self.gl.bind_buffer)(GL_SHADER_STORAGE_BUFFER, ssbo);
            (self.gl.buffer_data)(
                GL_SHADER_STORAGE_BUFFER,
                data.len() as isize,
                data.as_ptr() as *const c_void,
                GL_STATIC_DRAW,
            );
        }
    }
}

impl Drop for GpuRaytracer {
    fn drop(&mut self) {
        let buffers = [
            self.output_ssbo,
            self.frame_ssbo,
            self.sphere_ssbo,
            self.triangle_ssbo,
            self.area_light_ssbo,
            self.bvh_nodes_ssbo,
            self.bvh_prims_ssbo,
        ];
        unsafe {
            (self.gl.delete_buffers)(buffers.len() as i32, buffers.as_ptr());
            (self.gl.delete_program)(self.program);
            (self.gl.delete_shader)(self.shader);
        }
    }
}

fn denoise_bilateral(color: &mut [Vec3], width: usize, height: usize) {
    let src = color.to_vec();
    let sigma_color = 0.15_f64;
    let inv_sigma_c2 = 1.0 / (2.0 * sigma_color * sigma_color);
    for y in 0..height {
        for x in 0..width {
            let center = src[y * width + x];
            let mut sum = Vec3::ZERO;
            let mut weight_sum = 0.0_f64;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if nx < 0 || ny < 0 || nx >= width as i32 || ny >= height as i32 {
                        continue;
                    }
                    let s = src[ny as usize * width + nx as usize];
                    let dr = s.x - center.x;
                    let dg = s.y - center.y;
                    let db = s.z - center.z;
                    let dist2 = dr * dr + dg * dg + db * db;
                    let w_color = (-dist2 * inv_sigma_c2).exp();
                    let w_spatial = if dx == 0 && dy == 0 {
                        1.0
                    } else if dx.abs() + dy.abs() == 1 {
                        0.6
                    } else {
                        0.36
                    };
                    let w = w_color * w_spatial;
                    sum += s * w;
                    weight_sum += w;
                }
            }
            if weight_sum > 0.0 {
                color[y * width + x] = sum * (1.0 / weight_sum);
            }
        }
    }
}

#[cfg(target_os = "linux")]
mod desktop_factory {
    use super::{GpuRaytracer, c_void};
    use crate::api::display::{desktop_offscreen_context, DesktopOffscreenContext};

    /// Builds a GPU path tracer on a fresh hidden GLX 4.3 Pbuffer context.
    pub fn try_new_desktop(width: u32, height: u32) -> Result<(GpuRaytracer, DesktopOffscreenContext), String> {
        let ctx = desktop_offscreen_context(width.max(1), height.max(1), 4, 3)
            .ok_or_else(|| "no GLX 4.3+ offscreen context available".to_string())?;
        if !ctx.make_current() {
            return Err("glXMakeCurrent failed".into());
        }
        let loader = |name: &[u8]| -> *mut c_void { ctx.gl_get_proc(name) };
        let tracer = GpuRaytracer::new(&loader, false)?;
        Ok((tracer, ctx))
    }
}

#[cfg(target_os = "linux")]
pub use desktop_factory::try_new_desktop;

#[cfg(target_os = "android")]
mod android_factory {
    use super::{GpuRaytracer, c_void};
    use crate::api::display::NativeWindow;

    pub fn try_new_android(window: &NativeWindow) -> Result<GpuRaytracer, String> {
        if !window.has_backend() {
            return Err("native window has no GPU backend".into());
        }
        if !window.make_current() {
            return Err("eglMakeCurrent failed".into());
        }
        let loader = |name: &[u8]| -> *mut c_void { window.gl_get_proc(name) };
        GpuRaytracer::new(&loader, true)
    }
}

#[cfg(target_os = "android")]
pub use android_factory::try_new_android;

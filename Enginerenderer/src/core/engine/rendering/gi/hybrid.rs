use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::raytracing::ddgi::DdgiVolume;
use crate::core::engine::rendering::raster::pipeline::GBuffer;
use crate::core::engine::rendering::framebuffer::buffer::FrameBuffer;
use crate::core::engine::math::Mat4;

pub struct HybridGi;

impl HybridGi {
    pub fn apply(
        gbuffer: &GBuffer,
        ddgi: &DdgiVolume,
        inv_proj: &Mat4,
        inv_view: &Mat4,
        camera_pos: Vec3,
        fb: &mut FrameBuffer,
    ) {
        let w = gbuffer.width;
        let h = gbuffer.height;

        for y in 0..h {
            for x in 0..w {
                let idx = y * w + x;
                let depth = gbuffer.depth[idx];
                if depth >= 1.0 { continue; }

                let nx = (x as f32 / w as f32) * 2.0 - 1.0;
                let ny = 1.0 - (y as f32 / h as f32) * 2.0;
                let nz = (depth as f32) * 2.0 - 1.0;

                let clip = [nx, ny, nz, 1.0_f32];
                let view_space = mat4_transform(inv_proj, clip);
                let w_div = if view_space[3].abs() > f32::EPSILON { view_space[3] } else { 1.0 };
                let view_pos = [view_space[0] / w_div, view_space[1] / w_div, view_space[2] / w_div, 1.0];
                let world_space = mat4_transform(inv_view, view_pos);
                let world_pos = Vec3::new(world_space[0] as f64, world_space[1] as f64, world_space[2] as f64);

                let gn = gbuffer.normal[idx];
                let normal = gn.normalize();
                let view_dir = (camera_pos - world_pos).normalize();

                let irradiance = ddgi.sample_irradiance(world_pos, normal, view_dir, None);
                let albedo = gbuffer.albedo[idx];
                let albedo_f64 = Vec3::new(albedo[0], albedo[1], albedo[2]);

                let gi_contrib = Vec3::new(
                    irradiance.x * albedo_f64.x,
                    irradiance.y * albedo_f64.y,
                    irradiance.z * albedo_f64.z,
                );

                if idx < fb.color.len() {
                    fb.color[idx] += gi_contrib;
                }
            }
        }
    }
}

fn mat4_transform(m: &Mat4, v: [f32; 4]) -> [f32; 4] {
    [
        m.m[0][0]*v[0] + m.m[0][1]*v[1] + m.m[0][2]*v[2] + m.m[0][3]*v[3],
        m.m[1][0]*v[0] + m.m[1][1]*v[1] + m.m[1][2]*v[2] + m.m[1][3]*v[3],
        m.m[2][0]*v[0] + m.m[2][1]*v[1] + m.m[2][2]*v[2] + m.m[2][3]*v[3],
        m.m[3][0]*v[0] + m.m[3][1]*v[1] + m.m[3][2]*v[2] + m.m[3][3]*v[3],
    ]
}

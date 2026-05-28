//! GPU compute dispatch, framebuffer upload, fence/sync, and SIMD/GPU tag helpers.

use crate::core::engine::acces_hardware::{self, DrmDriver, KernelConfig};
use crate::core::engine::rendering::{
    framebuffer::FrameBuffer,
    raytracing::Scene,
    shader_dispatcher::TileComputeDescriptor,
};

use super::state::Renderer;

impl Renderer {
    pub(super) fn submit_compute_workload(&self, scene: &Scene, width: usize, height: usize) -> bool {
        let mut dispatcher = Self::lock_unpoisoned(&self.compute_dispatcher);
        if dispatcher.device_count() == 0 {
            return false;
        }

        let workgroup_size = acces_hardware::arch_optimal_workgroup().max(1);
        let kernel_x = (workgroup_size.min(8)) as u16;
        let kernel_y = workgroup_size.div_ceil(kernel_x as usize).max(1) as u16;
        let tile_size = (kernel_x as usize)
            .saturating_mul(kernel_y as usize)
            .max(16);
        let kernel = KernelConfig::new(kernel_x, kernel_y, 1)
            .with_shared_memory((workgroup_size * 32) as u32);
        let kernel_source = format!(
            "kernel trace_tile(u32 tile_id, u32 object_count, u32 triangle_count, u64 scene_signature) {{ u32 lanes = {}; u32 tiles_x = {}; u32 tiles_y = {}; }}",
            workgroup_size,
            width.div_ceil(tile_size),
            height.div_ceil(tile_size),
        );

        dispatcher
            .dispatch_tile_compute_with_kernel(TileComputeDescriptor {
                image_width: width,
                image_height: height,
                tile_size,
                config: kernel,
                kernel_name: "trace-tile",
                kernel_source: kernel_source.as_bytes(),
                scene_signature: scene.geometry_signature(),
                object_count: scene.objects.len(),
                triangle_count: scene.triangles.len(),
            })
            .is_ok()
    }

    pub(super) fn upload_framebuffer_to_gpu(&self, framebuffer: &FrameBuffer) -> bool {
        let Some(gpu) = self.gpu.as_ref() else {
            return false;
        };
        let mut rgb = vec![0u8; framebuffer.width * framebuffer.height * 3];
        for (pixel, out) in framebuffer.color.iter().zip(rgb.chunks_exact_mut(3)) {
            let corrected = pixel.clamp(0.0, 1.0).powf(1.0 / 2.2);
            out[0] = (corrected.x * 255.0).round() as u8;
            out[1] = (corrected.y * 255.0).round() as u8;
            out[2] = (corrected.z * 255.0).round() as u8;
        }
        gpu.write_framebuffer_rgb(&rgb)
    }

    pub(super) fn gpu_info_tag(&self) -> String {
        match &self.gpu {
            Some(g) if g.has_valid_metrics() => format!(
                "gpu(driver={} vram={}MB cu={} {:04x})",
                g.driver_name(),
                g.vram_bytes() / (1024 * 1024),
                g.compute_units(),
                g.info().device_id,
            ),
            Some(g) => format!("gpu(driver={} telemetry=unavailable)", g.driver_name()),
            None => "cpu-only".to_string(),
        }
    }

    pub(super) fn simd_tag(&self) -> &'static str {
        let s = &self.cpu_profile.simd_features;
        if s.avx512f { "AVX-512" }
        else if s.avx2 { "AVX2" }
        else if s.avx { "AVX" }
        else if s.fma { "FMA" }
        else if s.sse4_2 { "SSE4.2" }
        else if s.sse2 { "SSE2" }
        else if s.neon { "NEON" }
        else { "scalar" }
    }

    pub(super) fn gpu_fence_and_sync(&self) -> Option<f64> {
        if let Some(ref g) = self.gpu {
            let t0 = acces_hardware::precise_timestamp_ns();

            if g.has_active_framebuffer() && !matches!(g.driver(), DrmDriver::Radeon) {
                let nop_ib: [u32; 4] = [
                    0xC0021000,
                    0x00000000,
                    0x00000000,
                    0x00000000,
                ];
                match g.submit_ib(&nop_ib) {
                    Ok(cs_id) => {
                        g.sync_framebuffer();
                        let elapsed = acces_hardware::elapsed_ms(t0, acces_hardware::precise_timestamp_ns());
                        crate::runtime_log!("gpu: fence cs_id={} sync={:.2}ms", cs_id, elapsed);
                        return Some(elapsed);
                    }
                    Err(e) => {
                        crate::runtime_log!("gpu: fence submit failed ({}), fallback sync", e);
                    }
                }
            }

            g.sync_framebuffer();
            let elapsed = acces_hardware::elapsed_ms(t0, acces_hardware::precise_timestamp_ns());
            Some(elapsed)
        } else {
            None
        }
    }
}

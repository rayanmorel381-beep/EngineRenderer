use std::{
    fs::{self, File},
    io::{self, Write},
    path::Path,
    thread,
};

use crate::core::engine::acces_hardware::{
    self, precise_timestamp_ns, elapsed_ms as hw_elapsed_ms,
};


use crate::core::engine::acces_hardware::cpu::detect_core_frequencies;
use crate::core::engine::rendering::lod::manager::LodManager;
use crate::core::engine::rendering::lod::selection::LodSelection;
use crate::core::scheduler::adaptive::TileScheduler;

use super::acceleration::BvhNode;
use super::math::Vec3;
use super::primitives::{Ray, EPSILON};
use super::scene::Scene;
use super::shading::{
    luminance_estimate, make_seed, random_scalar, trace_ray, TraceContext,
};

#[derive(Debug, Clone, Copy)]
pub struct RenderConfig {
    pub width: usize,
    pub height: usize,
    pub base_samples_per_pixel: u32,
    pub max_bounces: u32,
    pub max_distance: f64,
    pub thread_count: usize,
    pub denoise_strength: f64,
    pub adaptive_sampling: bool,
    pub firefly_threshold: f64,
    pub denoise_radius: usize,
}

#[derive(Clone, Copy)]
struct PixelSampleContext<'a> {
    scene: &'a Scene,
    camera: &'a super::camera::Camera,
    config: &'a RenderConfig,
    lod_manager: &'a LodManager,
    bvh: Option<&'a BvhNode>,
}

#[derive(Debug, Clone)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<Vec3>,
}

impl Image {
    pub fn save_ppm<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let mut file = File::create(path)?;
        write!(file, "P6\n{} {}\n255\n", self.width, self.height)?;

        // Direct write — gamma correction + byte conversion in single pass
        let mut buffer = Vec::with_capacity(self.width * self.height * 3);
        for pixel in &self.pixels {
            let corrected = pixel.clamp(0.0, 1.0).powf(1.0 / 2.2);
            buffer.push((corrected.x * 255.0).round() as u8);
            buffer.push((corrected.y * 255.0).round() as u8);
            buffer.push((corrected.z * 255.0).round() as u8);
        }

        file.write_all(&buffer)
    }

    pub fn save_png<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let row_bytes = 1 + self.width * 3;
        let mut raw = vec![0u8; self.height * row_bytes];
        for y in 0..self.height {
            let row_start = y * row_bytes;
            for x in 0..self.width {
                let pixel = self.pixels[y * self.width + x];
                let c = pixel.clamp(0.0, 1.0).powf(1.0 / 2.2);
                let off = row_start + 1 + x * 3;
                raw[off] = (c.x * 255.0).round() as u8;
                raw[off + 1] = (c.y * 255.0).round() as u8;
                raw[off + 2] = (c.z * 255.0).round() as u8;
            }
        }

        let mut out = Vec::new();
        out.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);

        let mut ihdr = Vec::with_capacity(13);
        ihdr.extend_from_slice(&(self.width as u32).to_be_bytes());
        ihdr.extend_from_slice(&(self.height as u32).to_be_bytes());
        ihdr.push(8);
        ihdr.push(2);
        ihdr.push(0);
        ihdr.push(0);
        ihdr.push(0);
        png_write_chunk(&mut out, b"IHDR", &ihdr);
        png_write_chunk(&mut out, b"IDAT", &zlib_none(&raw));
        png_write_chunk(&mut out, b"IEND", &[]);

        let mut file = File::create(path)?;
        file.write_all(&out)
    }

    pub fn save_exr<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }

        let w = self.width as i32;
        let h = self.height as i32;

        let mut header = Vec::new();
        header.extend_from_slice(&[0x76, 0x2F, 0x31, 0x01]);
        header.extend_from_slice(&[2u8, 0, 0, 0]);

        exr_attr(
            &mut header,
            "channels",
            "chlist",
            &exr_chlist(&["B", "G", "R"]),
        );
        exr_attr(&mut header, "compression", "compression", &[0u8]);
        exr_attr(
            &mut header,
            "dataWindow",
            "box2i",
            &exr_box2i(0, 0, w - 1, h - 1),
        );
        exr_attr(
            &mut header,
            "displayWindow",
            "box2i",
            &exr_box2i(0, 0, w - 1, h - 1),
        );
        exr_attr(&mut header, "lineOrder", "lineOrder", &[0u8]);
        exr_attr(
            &mut header,
            "pixelAspectRatio",
            "float",
            &1.0f32.to_le_bytes(),
        );
        let v2f_zero = {
            let mut v = 0.0f32.to_le_bytes().to_vec();
            v.extend_from_slice(&0.0f32.to_le_bytes());
            v
        };
        exr_attr(&mut header, "screenWindowCenter", "v2f", &v2f_zero);
        exr_attr(
            &mut header,
            "screenWindowWidth",
            "float",
            &1.0f32.to_le_bytes(),
        );
        header.push(0u8);

        let scanline_body = 4 + 4 + 3 * self.width * 4;
        let data_start = header.len() + self.height * 8;

        let mut out = header;
        for y in 0..self.height {
            out.extend_from_slice(&((data_start + y * scanline_body) as u64).to_le_bytes());
        }
        for y in 0..self.height {
            out.extend_from_slice(&(y as i32).to_le_bytes());
            out.extend_from_slice(&((3 * self.width * 4) as u32).to_le_bytes());
            for x in 0..self.width {
                out.extend_from_slice(&(self.pixels[y * self.width + x].z as f32).to_le_bytes());
            }
            for x in 0..self.width {
                out.extend_from_slice(&(self.pixels[y * self.width + x].y as f32).to_le_bytes());
            }
            for x in 0..self.width {
                out.extend_from_slice(&(self.pixels[y * self.width + x].x as f32).to_le_bytes());
            }
        }

        let mut file = File::create(path)?;
        file.write_all(&out)
    }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        match path.as_ref().extension().and_then(|e| e.to_str()) {
            Some("png") => self.save_png(path),
            Some("exr") => self.save_exr(path),
            _ => self.save_ppm(path),
        }
    }
}

fn png_write_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let mut crc = 0xFFFF_FFFFu32;
    for &b in tag.iter().chain(data.iter()) {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    out.extend_from_slice(&(crc ^ 0xFFFF_FFFF).to_be_bytes());
}

fn zlib_none(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    if data.is_empty() {
        out.extend_from_slice(&[1u8, 0, 0, 0xFF, 0xFF]);
    } else {
        let mut rem = data;
        while !rem.is_empty() {
            let n = rem.len().min(65535);
            let last = n == rem.len();
            out.push(if last { 1 } else { 0 });
            out.extend_from_slice(&(n as u16).to_le_bytes());
            out.extend_from_slice(&(!(n as u16)).to_le_bytes());
            out.extend_from_slice(&rem[..n]);
            rem = &rem[n..];
        }
    }
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    out.extend_from_slice(&((b << 16) | a).to_be_bytes());
    out
}

fn exr_attr(out: &mut Vec<u8>, name: &str, ty: &str, data: &[u8]) {
    out.extend_from_slice(name.as_bytes());
    out.push(0);
    out.extend_from_slice(ty.as_bytes());
    out.push(0);
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(data);
}

fn exr_chlist(names: &[&str]) -> Vec<u8> {
    let mut data = Vec::new();
    for name in names {
        data.extend_from_slice(name.as_bytes());
        data.push(0);
        data.extend_from_slice(&2u32.to_le_bytes());
        data.push(0);
        data.extend_from_slice(&[0u8; 3]);
        data.extend_from_slice(&1i32.to_le_bytes());
        data.extend_from_slice(&1i32.to_le_bytes());
    }
    data.push(0);
    data
}

fn exr_box2i(xmin: i32, ymin: i32, xmax: i32, ymax: i32) -> Vec<u8> {
    let mut v = Vec::with_capacity(16);
    v.extend_from_slice(&xmin.to_le_bytes());
    v.extend_from_slice(&ymin.to_le_bytes());
    v.extend_from_slice(&xmax.to_le_bytes());
    v.extend_from_slice(&ymax.to_le_bytes());
    v
}

#[derive(Debug, Default, Clone, Copy)]
pub struct CpuRayTracer;

pub type BvhStats = super::acceleration::BvhStats;

impl CpuRayTracer {
    pub fn render(
        &self,
        scene: &Scene,
        camera: &super::camera::Camera,
        config: &RenderConfig,
        lod_manager: &LodManager,
    ) -> (Image, BvhStats) {
        let t_bvh = precise_timestamp_ns();
        let bvh = BvhNode::build(scene);
        let bvh_stats = bvh.as_ref().map(|n| n.stats()).unwrap_or_default();
        let bvh_ms = hw_elapsed_ms(t_bvh, precise_timestamp_ns());
        eprintln!("tracer: BVH build {:.2}ms (nodes={} leaves={})",
            bvh_ms, bvh_stats.node_count, bvh_stats.leaf_count);

        let core_freqs = detect_core_frequencies();
        if !core_freqs.is_empty() {
            let max_freq = core_freqs.iter().map(|c| c.frequency_hz).max().unwrap_or(0);
            let min_freq = core_freqs.iter().map(|c| c.frequency_hz).min().unwrap_or(0);
            eprintln!("tracer: {} cores, freq range {}-{}MHz",
                core_freqs.len(), min_freq / 1_000_000, max_freq / 1_000_000);
        }

        self.render_with_bvh(scene, camera, config, lod_manager, bvh.as_ref())
    }

    pub fn render_with_scheduler(
        &self,
        scene: &Scene,
        camera: &super::camera::Camera,
        config: &RenderConfig,
        lod_manager: &LodManager,
        bvh: Option<&BvhNode>,
        scheduler: &TileScheduler,
    ) -> (Image, BvhStats) {
        let t_start = precise_timestamp_ns();
        let bvh_stats = bvh.map(|n| n.stats()).unwrap_or_default();

        let t_dispatch = precise_timestamp_ns();
        let (tile_results, sched_report) = scheduler.dispatch(|tile| {
            let mut pixels = Vec::with_capacity(tile.width * tile.height);
            let context = PixelSampleContext {
                scene,
                camera,
                config,
                lod_manager,
                bvh,
            };
            for dy in 0..tile.height {
                for dx in 0..tile.width {
                    pixels.push(self.render_pixel(context, tile.x + dx, tile.y + dy));
                }
            }
            pixels
        });
        sched_report.log_summary();
        let dispatch_ms = hw_elapsed_ms(t_dispatch, precise_timestamp_ns());

        let t_assemble = precise_timestamp_ns();
        let mut pixels = vec![Vec3::ZERO; config.width * config.height];
        for (idx, chunk) in tile_results {
            let tile = scheduler.tile_at(idx);
            for dy in 0..tile.height {
                let src_start = dy * tile.width;
                let dst_start = (tile.y + dy) * config.width + tile.x;
                let row_len = tile.width;
                pixels[dst_start..dst_start + row_len]
                    .copy_from_slice(&chunk[src_start..src_start + row_len]);
            }
        }
        let assemble_ms = hw_elapsed_ms(t_assemble, precise_timestamp_ns());

        let image = Image {
            width: config.width,
            height: config.height,
            pixels,
        };

        let t_denoise = precise_timestamp_ns();
        let denoised = self.parallel_denoise(
            image,
            config.denoise_strength,
            config.denoise_radius,
            config.firefly_threshold,
            config.thread_count,
        );
        let denoise_ms = hw_elapsed_ms(t_denoise, precise_timestamp_ns());

        let total_ms = hw_elapsed_ms(t_start, precise_timestamp_ns());
        eprintln!(
            "tracer: total={:.1}ms (dispatch={:.1} assemble={:.1} denoise={:.1})",
            total_ms, dispatch_ms, assemble_ms, denoise_ms,
        );

        (denoised, bvh_stats)
    }

    pub fn render_with_bvh(
        &self,
        scene: &Scene,
        camera: &super::camera::Camera,
        config: &RenderConfig,
        lod_manager: &LodManager,
        bvh: Option<&BvhNode>,
    ) -> (Image, BvhStats) {
        let t_start = precise_timestamp_ns();
        let bvh_stats = bvh.map(|n| n.stats()).unwrap_or_default();

        let t_dispatch = precise_timestamp_ns();
        let scheduler = TileScheduler::new(config.width, config.height, config.thread_count);

        let (tile_results, sched_report) = scheduler.dispatch(|tile| {
            let mut pixels = Vec::with_capacity(tile.width * tile.height);
            let context = PixelSampleContext {
                scene,
                camera,
                config,
                lod_manager,
                bvh,
            };
            for dy in 0..tile.height {
                for dx in 0..tile.width {
                    pixels.push(self.render_pixel(context, tile.x + dx, tile.y + dy));
                }
            }
            pixels
        });
        sched_report.log_summary();
        let dispatch_ms = hw_elapsed_ms(t_dispatch, precise_timestamp_ns());

        let t_assemble = precise_timestamp_ns();
        let mut pixels = vec![Vec3::ZERO; config.width * config.height];
        for (idx, chunk) in tile_results {
            let tile = scheduler.tile_at(idx);
            for dy in 0..tile.height {
                let src_start = dy * tile.width;
                let dst_start = (tile.y + dy) * config.width + tile.x;
                let row_len = tile.width;
                pixels[dst_start..dst_start + row_len]
                    .copy_from_slice(&chunk[src_start..src_start + row_len]);
            }
        }
        let assemble_ms = hw_elapsed_ms(t_assemble, precise_timestamp_ns());

        let image = Image {
            width: config.width,
            height: config.height,
            pixels,
        };

        let t_denoise = precise_timestamp_ns();
        let denoised = self.parallel_denoise(
            image,
            config.denoise_strength,
            config.denoise_radius,
            config.firefly_threshold,
            config.thread_count,
        );
        let denoise_ms = hw_elapsed_ms(t_denoise, precise_timestamp_ns());

        let total_ms = hw_elapsed_ms(t_start, precise_timestamp_ns());
        eprintln!(
            "tracer: total={:.1}ms (dispatch={:.1} assemble={:.1} denoise={:.1})",
            total_ms, dispatch_ms, assemble_ms, denoise_ms,
        );

        (denoised, bvh_stats)
    }

    fn render_pixel(&self, context: PixelSampleContext<'_>, x: usize, y: usize) -> Vec3 {
        let center_u = (x as f64 + 0.5) / context.config.width as f64;
        let center_v = 1.0 - (y as f64 + 0.5) / context.config.height as f64;
        let center_bias = (1.0 - (((center_u - 0.5).abs() + (center_v - 0.5).abs()) * 1.35)).clamp(0.0, 1.0);
        let distance_hint = context.camera.focus_distance() * (1.25 - center_bias * 0.35);
        let radius_hint = 0.40 + center_bias * 1.40;
        let lod = if context.config.adaptive_sampling {
            context.lod_manager.select(distance_hint.max(0.001), radius_hint)
        } else {
            LodSelection::background()
        };

        let base_samples = context.config.base_samples_per_pixel.max(1) as usize;
        let target_samples = if context.config.adaptive_sampling {
            let probe_ray = context.camera.ray(center_u, center_v);
            self.adaptive_sample_count(
                context.scene,
                context.config,
                lod,
                probe_ray,
                base_samples,
                context.bvh,
            )
        } else {
            base_samples
        };
        let min_samples = target_samples.clamp(1, 8);

        let mut color = Vec3::ZERO;
        let mut used_samples = 0usize;
        let mut mean_luma = 0.0_f64;
        let mut m2 = 0.0_f64;

        for sample_idx in 0..target_samples {
            let seed = make_seed(x as u32, y as u32, sample_idx as u32);
            let phase = random_scalar(seed ^ 0xD1B5_4A35);
            let jitter_u = (random_scalar(seed ^ 0xA53C_9E2D) - 0.5) / context.config.width as f64;
            let jitter_v = (random_scalar(seed ^ 0x7F4A_7C15) - 0.5) / context.config.height as f64;

            let temporal_offset_u = (phase - 0.5) * 0.45 / context.config.width as f64;
            let temporal_offset_v = (0.5 - phase) * 0.45 / context.config.height as f64;

            let u = (center_u + temporal_offset_u + jitter_u).clamp(0.0, 1.0);
            let v = (center_v + temporal_offset_v + jitter_v).clamp(0.0, 1.0);
            let lens_u = random_scalar(seed ^ 0x91E1_0DA5);
            let lens_v = random_scalar(seed ^ 0xC2B3_A13F);
            let shutter_t = (phase + random_scalar(seed ^ 0x27D4_EB2D) * 0.25).fract();
            let ray = context.camera.ray_with_lens(u, v, lens_u, lens_v, shutter_t);
            let sample = self.limit_fireflies(
                trace_ray(ray, 0, TraceContext {
                    scene: context.scene,
                    lod_manager: context.lod_manager,
                    global_bounce_limit: context.config.max_bounces,
                    seed,
                    bvh: context.bvh,
                }),
                context.config.firefly_threshold,
            );
            color += sample;
            used_samples += 1;

            let luma = luminance_estimate(sample);
            let delta = luma - mean_luma;
            mean_luma += delta / used_samples as f64;
            let delta2 = luma - mean_luma;
            m2 += delta * delta2;

            if context.config.adaptive_sampling && used_samples >= min_samples && used_samples.is_multiple_of(4) {
                let variance = if used_samples > 1 {
                    m2 / (used_samples - 1) as f64
                } else {
                    0.0
                };
                let relative_noise = variance.sqrt() / mean_luma.max(0.04);
                let threshold = if center_bias > 0.70 {
                    0.050
                } else if center_bias > 0.35 {
                    0.038
                } else {
                    0.028
                };
                if relative_noise <= threshold {
                    break;
                }
            }
        }

        color / used_samples.max(1) as f64
    }

    fn adaptive_sample_count(
        &self,
        scene: &Scene,
        config: &RenderConfig,
        lod: LodSelection,
        probe_ray: Ray,
        base_samples: usize,
        bvh: Option<&BvhNode>,
    ) -> usize {
        if !config.adaptive_sampling {
            return base_samples.max(1);
        }

        let material_complexity = BvhNode::hit_scene(scene, &probe_ray, EPSILON, config.max_distance, bvh)
            .map(|hit| {
                0.12
                    + hit.material.reflectivity * 0.45
                    + hit.material.transmission * 0.55
                    + hit.material.clearcoat * 0.20
                    + hit.material.iridescence * 0.25
                    + hit.material.anisotropy * 0.22
                    + hit.material.subsurface * 0.18
            })
            .unwrap_or(0.08);

        let lod_boost = (lod.reflection_boost - 1.0).max(0.0) * 0.22
            + (lod.texture_frequency - 1.0).max(0.0) * 0.04;
        let multiplier = (0.82 + material_complexity * 0.55 + lod_boost).clamp(0.85, 1.35);

        ((base_samples.max(1) as f64) * multiplier).round() as usize
    }

    fn limit_fireflies(&self, sample: Vec3, threshold: f64) -> Vec3 {
        let threshold = threshold.max(0.5);
        let luma = luminance_estimate(sample);
        if luma <= threshold {
            sample
        } else {
            sample * (threshold / luma)
        }
    }

    /// Parallelized denoise — splits the image into horizontal bands
    /// and processes them across worker threads pinned to physical cores.
    fn parallel_denoise(
        &self,
        image: Image,
        strength: f64,
        radius: usize,
        firefly_threshold: f64,
        thread_count: usize,
    ) -> Image {
        let strength = strength.clamp(0.0, 1.0);
        if strength <= f64::EPSILON || image.width == 0 || image.height == 0 {
            return image;
        }

        let radius = radius.clamp(1, 3) as isize;
        let firefly_threshold = firefly_threshold.max(1.1);
        let w = image.width;
        let h = image.height;
        let mut current = image.pixels;
        let mut scratch = vec![Vec3::ZERO; current.len()];

        // ── Pass 1: Parallel firefly clamping ───────────────────────
        {
            let workers = thread_count.min(h).max(1);
            let band_h = h.div_ceil(workers);

            thread::scope(|scope| {
                let source: &[Vec3] = &current;
                let out_chunks = scratch.chunks_mut(band_h * w);

                for (worker_id, out_band) in out_chunks.enumerate() {
                    let y_start = worker_id * band_h;
                    let y_end = (y_start + band_h).min(h);
                    if y_start >= y_end { continue; }

                    scope.spawn(move || {
                        acces_hardware::pin_thread_to_core(worker_id);
                        for y in y_start..y_end {
                            for x in 0..w {
                                let index = y * w + x;
                                let local_index = (y - y_start) * w + x;
                                let center = source[index];
                                let center_luma = luminance_estimate(center);
                                let mut neighborhood = Vec3::ZERO;
                                let mut count = 0.0_f64;

                                for oy in -1isize..=1 {
                                    for ox in -1isize..=1 {
                                        if ox == 0 && oy == 0 { continue; }
                                        let sx = x as isize + ox;
                                        let sy = y as isize + oy;
                                        if sx < 0 || sy < 0 || sx >= w as isize || sy >= h as isize { continue; }
                                        neighborhood += source[sy as usize * w + sx as usize];
                                        count += 1.0;
                                    }
                                }

                                let result = if count > 0.0 {
                                    let mean = neighborhood / count;
                                    let mean_luma = luminance_estimate(mean).max(0.001);
                                    if center_luma > mean_luma * firefly_threshold {
                                        let f = (mean_luma * firefly_threshold / center_luma).clamp(0.0, 1.0);
                                        center * f + mean * (1.0 - f)
                                    } else {
                                        center
                                    }
                                } else {
                                    center
                                };
                                out_band[local_index] = result;
                            }
                        }
                    });
                }
            });
        }
        std::mem::swap(&mut current, &mut scratch);

        // ── Pass 2+3: Parallel bilateral denoise ────────────────────
        let pass_count = if strength > 0.55 { 2 } else { 1 };
        for pass_index in 0..pass_count {
            let workers = thread_count.min(h).max(1);
            let band_h = h.div_ceil(workers);

            thread::scope(|scope| {
                let source: &[Vec3] = &current;
                let out_chunks = scratch.chunks_mut(band_h * w);

                for (worker_id, out_band) in out_chunks.enumerate() {
                    let y_start = worker_id * band_h;
                    let y_end = (y_start + band_h).min(h);
                    if y_start >= y_end { continue; }

                    scope.spawn(move || {
                        acces_hardware::pin_thread_to_core(worker_id);
                        for y in y_start..y_end {
                            for x in 0..w {
                                let index = y * w + x;
                                let local_index = (y - y_start) * w + x;
                                let center = source[index];
                                let center_luma = luminance_estimate(center);
                                let mut accumulated = center;
                                let mut total_weight = 1.0_f64;

                                for oy in -radius..=radius {
                                    for ox in -radius..=radius {
                                        if ox == 0 && oy == 0 { continue; }
                                        let sx = x as isize + ox;
                                        let sy = y as isize + oy;
                                        if sx < 0 || sy < 0 || sx >= w as isize || sy >= h as isize { continue; }

                                        let sample = source[sy as usize * w + sx as usize];
                                        let sample_luma = luminance_estimate(sample);
                                        let luma_delta = (sample_luma - center_luma).abs();
                                        let color_delta = (sample - center).length();
                                        let dist2 = (ox * ox + oy * oy) as f64;
                                        let spatial = 1.0 / (1.0 + dist2 * (0.65 + pass_index as f64 * 0.20));
                                        let edge = 1.0 / (1.0 + luma_delta * 11.0 + color_delta * 4.5);
                                        let highlight = if sample_luma > center_luma * firefly_threshold { 0.22 } else { 1.0 };
                                        let weight = strength * spatial * edge * highlight;
                                        accumulated += sample * weight;
                                        total_weight += weight;
                                    }
                                }

                                let blurred = accumulated / total_weight.max(1.0);
                                let detail = (center - blurred).length();
                                let preserve = (1.0 / (1.0 + detail * 4.5)).clamp(0.18, 0.92);
                                out_band[local_index] = center.lerp(blurred, strength * preserve);
                            }
                        }
                    });
                }
            });

            std::mem::swap(&mut current, &mut scratch);
        }

        Image {
            width: w,
            height: h,
            pixels: current,
        }
    }
}

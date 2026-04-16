use std::{error::Error, path::PathBuf};

use crate::api::scenes::SceneDescriptor;
use crate::core::engine::rendering::renderer::{Renderer, types::RenderPreset};
use crate::core::engine::rendering::raytracing::{Camera, Vec3};
use crate::core::engine::rendering::raytracing::acceleration::BvhNode;
use crate::core::scheduler::adaptive::TileScheduler;

use super::clip::AnimationClip;

/// Résultat de rendu d'une frame.
#[derive(Debug, Clone)]
pub struct FrameResult {
    /// Index de frame.
    pub frame:       usize,
    /// Temps de frame en secondes.
    pub time_secs:   f64,
    /// Fichier de sortie.
    pub output_path: PathBuf,
    /// Durée en millisecondes.
    pub duration_ms: u128,
}

/// Résultat global d'une séquence rendue.
#[derive(Debug, Clone)]
pub struct SequenceResult {
    /// Liste des frames rendues.
    pub frames:      Vec<FrameResult>,
    /// Durée totale en ms.
    pub total_ms:    u128,
    /// Dossier de sortie.
    pub output_dir:  PathBuf,
    /// Nombre de frames attendu.
    pub frame_count: usize,
    /// FPS cible.
    pub fps:         f64,
}

impl SequenceResult {
    /// Durée moyenne par frame en ms.
    pub fn average_frame_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.total_ms as f64 / self.frames.len() as f64
    }
}

/// Orchestrateur de rendu de séquence image par image.
pub struct FrameSequencer {
    /// Scène de base.
    pub base:         SceneDescriptor,
    /// Clip d'animation.
    pub clip:         AnimationClip,
    /// Dossier de sortie.
    pub output_dir:   PathBuf,
    /// Préfixe des fichiers frame.
    pub frame_prefix: String,
    /// Preset de rendu.
    pub preset:       RenderPreset,
    /// Largeur de rendu.
    pub width:        usize,
    /// Hauteur de rendu.
    pub height:       usize,
}

impl FrameSequencer {
    /// Crée un séquenceur complet.
    pub fn new(
        base:         SceneDescriptor,
        clip:         AnimationClip,
        output_dir:   impl Into<PathBuf>,
        frame_prefix: impl Into<String>,
        preset:       RenderPreset,
        width:        usize,
        height:       usize,
    ) -> Self {
        Self {
            base,
            clip,
            output_dir: output_dir.into(),
            frame_prefix: frame_prefix.into(),
            preset,
            width,
            height,
        }
    }

    /// Rend toute la séquence sur disque.
    pub fn render_all(&self) -> Result<SequenceResult, Box<dyn Error>> {
        use crate::core::engine::acces_hardware::{precise_timestamp_ns, elapsed_ms as hw_elapsed};

        let frame_count = self.clip.frame_count();
        let renderer = Renderer::with_resolution(self.width, self.height);
        let mut frames = Vec::with_capacity(frame_count);
        let t_total = precise_timestamp_ns();

        std::fs::create_dir_all(&self.output_dir)?;

        let ext = detect_ext(&self.frame_prefix);
        let prefix = stem(&self.frame_prefix);
        let aspect = self.width as f64 / self.height as f64;

        let (mut base_scene, _) = self.base.clone().into_builder().build(aspect);

        let t_bvh = precise_timestamp_ns();
        let bvh = BvhNode::build(&base_scene);
        let bvh_ms = hw_elapsed(t_bvh, precise_timestamp_ns());
        eprintln!("animation: BVH cached in {:.2}ms for {} frames", bvh_ms, frame_count);

        let config = renderer.config_for(self.preset);
        let pixel_work = config.width * config.height * config.base_samples_per_pixel as usize;
        let max_threads = renderer.hw_caps.optimal_render_threads_for_input(pixel_work);
        let logical_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(max_threads)
            .max(1);
        let thread_count = logical_threads.max(max_threads).min(16);
        let scheduler = TileScheduler::new(self.width, self.height, thread_count);
        eprintln!(
            "animation: scheduler cached — {} threads/{} for {} frames",
            thread_count, max_threads, frame_count,
        );

        for idx in 0..frame_count {
            let time = self.clip.time_for_frame(idx);

            let cam_frame = self.clip.camera.as_ref().and_then(|tl| tl.sample(time));
            let camera = if let Some(cf) = cam_frame {
                Camera::look_at(
                    Vec3::new(cf.eye[0], cf.eye[1], cf.eye[2]),
                    Vec3::new(cf.target[0], cf.target[1], cf.target[2]),
                    Vec3::new(0.0, 1.0, 0.0),
                    cf.fov_degrees,
                    aspect,
                )
            } else {
                Camera::look_at(
                    Vec3::new(self.base.camera.eye[0], self.base.camera.eye[1], self.base.camera.eye[2]),
                    Vec3::new(self.base.camera.target[0], self.base.camera.target[1], self.base.camera.target[2]),
                    Vec3::new(0.0, 1.0, 0.0),
                    self.base.camera.fov_degrees,
                    aspect,
                )
            };

            if let Some(sf) = self.clip.sun.as_ref().and_then(|tl| tl.sample(time)) {
                base_scene.sun.direction = Vec3::new(sf.direction[0], sf.direction[1], sf.direction[2]);
                base_scene.sun.color = Vec3::new(sf.color[0], sf.color[1], sf.color[2]);
                base_scene.sun.intensity = sf.intensity;
            }
            if let Some(sk) = self.clip.sky.as_ref().and_then(|tl| tl.sample(time)) {
                base_scene.sky_top = Vec3::new(sk.top[0], sk.top[1], sk.top[2]);
                base_scene.sky_bottom = Vec3::new(sk.bottom[0], sk.bottom[1], sk.bottom[2]);
            }
            if let Some(e) = self.clip.exposure.as_ref().and_then(|tl| tl.sample(time)) {
                base_scene.exposure = e;
            }

            let file_name = format!("{prefix}_{idx:05}.{ext}");
            let output_path = self.output_dir.join(&file_name);

            let t_frame = precise_timestamp_ns();
            let report = renderer.render_animation_frame(
                &base_scene, &camera, bvh.as_ref(), &scheduler, &output_path, self.preset,
            )?;
            let frame_ms = hw_elapsed(t_frame, precise_timestamp_ns());

            eprintln!("animation: frame {}/{} t={:.3}s → {} ({:.1}ms)",
                idx + 1, frame_count, time, output_path.display(), frame_ms);

            frames.push(FrameResult {
                frame: idx,
                time_secs: time,
                output_path,
                duration_ms: report.duration_ms,
            });
        }

        let total_ms = hw_elapsed(t_total, precise_timestamp_ns()) as u128;

        Ok(SequenceResult {
            frames,
            total_ms,
            output_dir: self.output_dir.clone(),
            frame_count,
            fps: self.clip.fps,
        })
    }

    /// Rend toute la séquence en fenêtre native.
    pub fn render_all_to_window(&self) -> Result<SequenceResult, Box<dyn Error>> {
        use crate::core::engine::acces_hardware::{
            precise_timestamp_ns, elapsed_ms as hw_elapsed,
            NativeWindow,
        };

        let frame_count = self.clip.frame_count();
        let mut frames = Vec::with_capacity(frame_count);
        let t_total = precise_timestamp_ns();
        let output_width = self.width;
        let output_height = self.height;
        let target_fps = self.clip.fps.max(1.0);
        let frame_budget_ns = (1_000_000_000.0 / target_fps) as u64;
        let frame_budget_ms = 1000.0 / target_fps;
        let realtime_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .clamp(1, 16);
        let initial_scale = (30.0 / target_fps).sqrt().clamp(0.18, 1.0);
        let mut internal_width = ((output_width as f64) * initial_scale).round() as usize;
        let mut internal_height = ((output_height as f64) * initial_scale).round() as usize;
        internal_width = internal_width.max(160).min(output_width.max(160));
        internal_height = internal_height.max(90).min(output_height.max(90));
        let mut renderer = Renderer::with_resolution(internal_width, internal_height);

        let aspect = output_width as f64 / output_height as f64;
        let (mut base_scene, _) = self.base.clone().into_builder().build(aspect);

        let t_bvh = precise_timestamp_ns();
        let bvh = BvhNode::build(&base_scene);
        let bvh_ms = hw_elapsed(t_bvh, precise_timestamp_ns());
        eprintln!("window: BVH cached in {:.2}ms for {} frames", bvh_ms, frame_count);

        let mut scheduler = TileScheduler::new(internal_width, internal_height, realtime_threads);
        eprintln!(
            "window: scheduler cached — {} threads for {} frames",
            realtime_threads, frame_count,
        );

        let title = format!("EngineRenderer — {}x{} @ {}fps", output_width, output_height, self.clip.fps as u32);
        let mut window = NativeWindow::open(output_width, output_height, &title);
        if window.is_none() {
            eprintln!("window: display unavailable, falling back to disk render");
            return self.render_all();
        }
        let window = window.as_mut().unwrap();

        for idx in 0..frame_count {
            if window.should_close() {
                eprintln!("window: closed by user at frame {}", idx);
                break;
            }

            let time = self.clip.time_for_frame(idx);

            let cam_frame = self.clip.camera.as_ref().and_then(|tl| tl.sample(time));
            let camera = if let Some(cf) = cam_frame {
                Camera::look_at(
                    Vec3::new(cf.eye[0], cf.eye[1], cf.eye[2]),
                    Vec3::new(cf.target[0], cf.target[1], cf.target[2]),
                    Vec3::new(0.0, 1.0, 0.0),
                    cf.fov_degrees,
                    aspect,
                )
            } else {
                Camera::look_at(
                    Vec3::new(self.base.camera.eye[0], self.base.camera.eye[1], self.base.camera.eye[2]),
                    Vec3::new(self.base.camera.target[0], self.base.camera.target[1], self.base.camera.target[2]),
                    Vec3::new(0.0, 1.0, 0.0),
                    self.base.camera.fov_degrees,
                    aspect,
                )
            };

            if let Some(sf) = self.clip.sun.as_ref().and_then(|tl| tl.sample(time)) {
                base_scene.sun.direction = Vec3::new(sf.direction[0], sf.direction[1], sf.direction[2]);
                base_scene.sun.color = Vec3::new(sf.color[0], sf.color[1], sf.color[2]);
                base_scene.sun.intensity = sf.intensity;
            }
            if let Some(sk) = self.clip.sky.as_ref().and_then(|tl| tl.sample(time)) {
                base_scene.sky_top = Vec3::new(sk.top[0], sk.top[1], sk.top[2]);
                base_scene.sky_bottom = Vec3::new(sk.bottom[0], sk.bottom[1], sk.bottom[2]);
            }
            if let Some(e) = self.clip.exposure.as_ref().and_then(|tl| tl.sample(time)) {
                base_scene.exposure = e;
            }

            let t_frame = precise_timestamp_ns();
            let (color, report) = renderer.render_animation_frame_to_buffer(
                &base_scene, &camera, bvh.as_ref(), &scheduler, self.preset,
            )?;
            let frame_ms = hw_elapsed(t_frame, precise_timestamp_ns());

            let argb = upscale_argb_from_vec3(
                &color,
                report.width,
                report.height,
                output_width,
                output_height,
            );
            window.present_frame(&argb, output_width, output_height);

            eprintln!("window: frame {}/{} t={:.3}s ({:.1}ms)",
                idx + 1, frame_count, time, frame_ms);

            if frame_ms > frame_budget_ms * 1.15 && internal_width > 160 && internal_height > 90 {
                internal_width = ((internal_width as f64) * 0.82).round() as usize;
                internal_height = ((internal_height as f64) * 0.82).round() as usize;
                internal_width = internal_width.max(160).min(output_width.max(160));
                internal_height = internal_height.max(90).min(output_height.max(90));
                renderer = Renderer::with_resolution(internal_width, internal_height);
                scheduler = TileScheduler::new(internal_width, internal_height, realtime_threads);
            } else if frame_ms < frame_budget_ms * 0.60
                && internal_width < output_width
                && internal_height < output_height
            {
                internal_width = ((internal_width as f64) * 1.10).round() as usize;
                internal_height = ((internal_height as f64) * 1.10).round() as usize;
                internal_width = internal_width.max(160).min(output_width.max(160));
                internal_height = internal_height.max(90).min(output_height.max(90));
                renderer = Renderer::with_resolution(internal_width, internal_height);
                scheduler = TileScheduler::new(internal_width, internal_height, realtime_threads);
            }

            let elapsed_ns = precise_timestamp_ns() - t_frame;
            if elapsed_ns < frame_budget_ns {
                std::thread::sleep(std::time::Duration::from_nanos(frame_budget_ns - elapsed_ns));
            }

            frames.push(FrameResult {
                frame: idx,
                time_secs: time,
                output_path: PathBuf::new(),
                duration_ms: report.duration_ms,
            });
        }

        let total_ms = hw_elapsed(t_total, precise_timestamp_ns()) as u128;

        Ok(SequenceResult {
            frames,
            total_ms,
            output_dir: self.output_dir.clone(),
            frame_count,
            fps: self.clip.fps,
        })
    }
}

fn detect_ext(prefix: &str) -> &str {
    if prefix.ends_with(".png") { "png" }
    else if prefix.ends_with(".exr") { "exr" }
    else { "png" }
}

fn stem(prefix: &str) -> &str {
    if let Some(dot) = prefix.rfind('.') { &prefix[..dot] } else { prefix }
}

fn upscale_argb_from_vec3(
    pixels: &[Vec3],
    src_width: usize,
    src_height: usize,
    dst_width: usize,
    dst_height: usize,
) -> Vec<u8> {
    let mut out = vec![0u8; dst_width.saturating_mul(dst_height).saturating_mul(4)];
    let clamp = |v: f64| -> u8 { (v.clamp(0.0, 1.0) * 255.0).round() as u8 };
    let max_x = src_width.saturating_sub(1);
    let max_y = src_height.saturating_sub(1);

    for y in 0..dst_height {
        let sy = y.saturating_mul(src_height).saturating_div(dst_height.max(1)).min(max_y);
        for x in 0..dst_width {
            let sx = x.saturating_mul(src_width).saturating_div(dst_width.max(1)).min(max_x);
            let src_idx = sy.saturating_mul(src_width).saturating_add(sx);
            let dst_idx = y.saturating_mul(dst_width).saturating_add(x).saturating_mul(4);
            let p = pixels.get(src_idx).copied().unwrap_or(Vec3::ZERO);
            out[dst_idx] = 255;
            out[dst_idx + 1] = clamp(p.x);
            out[dst_idx + 2] = clamp(p.y);
            out[dst_idx + 3] = clamp(p.z);
        }
    }

    out
}

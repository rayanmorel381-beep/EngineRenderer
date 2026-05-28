use std::{error::Error, path::PathBuf};

use crate::api::scenes::SceneDescriptor;
use crate::core::debug::profiling::format_adaptation;
use crate::core::debug::runtime::RuntimeAdaptationState;
use crate::core::engine::rendering::renderer::{Renderer, types::RenderPreset};
use crate::core::engine::rendering::raytracing::{Camera, Vec3};
use crate::core::engine::rendering::raytracing::acceleration::BvhNode;
use crate::core::scheduler::adaptive::{SchedulerTuning, TileScheduler};

use super::clip::AnimationClip;

/// Per-frame rendering metadata.
#[derive(Debug, Clone)]
pub struct FrameResult {
    /// Zero-based frame index.
    pub frame:       usize,
    /// Frame timestamp in seconds.
    pub time_secs:   f64,
    /// Output file path for the rendered frame.
    pub output_path: PathBuf,
    /// Frame render time in milliseconds.
    pub duration_ms: u128,
}

/// Result summary for a full frame sequence render.
#[derive(Debug, Clone)]
pub struct SequenceResult {
    /// Collected per-frame results.
    pub frames:      Vec<FrameResult>,
    /// Total render time in milliseconds.
    pub total_ms:    u128,
    /// Output directory containing frames.
    pub output_dir:  PathBuf,
    /// Number of frames requested.
    pub frame_count: usize,
    /// Sequence frame rate.
    pub fps:         f64,
}

impl SequenceResult {
    /// Returns the average frame time in milliseconds.
    pub fn average_frame_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.total_ms as f64 / self.frames.len() as f64
    }
}

/// Renders an AnimationClip into an image sequence.
pub struct FrameSequencer {
    /// Base scene descriptor used as the animation source.
    pub base:         SceneDescriptor,
    /// Animation clip driving animated properties.
    pub clip:         AnimationClip,
    /// Directory where frame images are written.
    pub output_dir:   PathBuf,
    /// File prefix used for frame names.
    pub frame_prefix: String,
    /// Rendering preset used for each frame.
    pub preset:       RenderPreset,
    /// Output width in pixels.
    pub width:        usize,
    /// Output height in pixels.
    pub height:       usize,
}

impl FrameSequencer {
    /// Creates a frame sequencer from scene, clip, and output settings.
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

    /// Renders the full sequence to image files on disk.
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
        crate::runtime_log!("animation: BVH cached in {:.2}ms for {} frames", bvh_ms, frame_count);

        let config = renderer.config_for(self.preset);
        let pixel_work = config.width * config.height * config.base_samples_per_pixel as usize;
        let max_threads = renderer.hw_caps.optimal_render_threads_for_input(pixel_work);
        let logical_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(max_threads)
            .max(1);
        let thread_count = logical_threads.max(max_threads).min(16);
        let scheduler = TileScheduler::new_tuned(
            self.width,
            self.height,
            thread_count,
            SchedulerTuning::new(1.0 + (config.base_samples_per_pixel as f64).sqrt() * 0.15),
        );
        crate::runtime_log!(
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

            crate::runtime_log!("animation: frame {}/{} t={:.3}s → {} ({:.1}ms)",
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

    /// Renders the sequence to a realtime window and optional frame outputs.
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
        let mut sample_pressure_scale = 1.0_f64;
        let mut resize_cooldown_frames = 0usize;
        let mut over_budget_streak = 0usize;
        let mut under_budget_streak = 0usize;

        let aspect = output_width as f64 / output_height as f64;
        let (mut base_scene, _) = self.base.clone().into_builder().build(aspect);

        let t_bvh = precise_timestamp_ns();
        let bvh = BvhNode::build(&base_scene);
        let bvh_ms = hw_elapsed(t_bvh, precise_timestamp_ns());
        crate::runtime_log!("window: BVH cached in {:.2}ms for {} frames", bvh_ms, frame_count);

        let mut scheduler_tuning = SchedulerTuning::default();
        let mut scheduler = TileScheduler::new_tuned(
            internal_width,
            internal_height,
            realtime_threads,
            scheduler_tuning,
        );
        crate::runtime_log!(
            "window: scheduler cached — {} threads for {} frames",
            realtime_threads, frame_count,
        );

        let title = format!("EngineRenderer — {}x{} @ {}fps", output_width, output_height, self.clip.fps as u32);
        let mut window = NativeWindow::open(output_width, output_height, &title);
        if window.is_none() {
            crate::runtime_log!("window: display unavailable, falling back to disk render");
            return self.render_all();
        }
        let Some(window) = window.as_mut() else {
            return self.render_all();
        };

        for idx in 0..frame_count {
            if window.should_close() {
                crate::runtime_log!("window: closed by user at frame {}", idx);
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
            let (color, report) = renderer.render_animation_frame_to_buffer_with_pressure(
                &base_scene, &camera, bvh.as_ref(), &scheduler, self.preset,
                sample_pressure_scale,
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

            crate::runtime_log!("window: frame {}/{} t={:.3}s ({:.1}ms)",
                idx + 1, frame_count, time, frame_ms);

            let target_pressure_scale = (frame_budget_ms / frame_ms.max(1.0)).clamp(0.55, 1.10);
            sample_pressure_scale = smooth_runtime_pressure(sample_pressure_scale, target_pressure_scale);
            scheduler_tuning = SchedulerTuning::new(smooth_runtime_granularity(
                scheduler_tuning.granularity_bias(),
                SchedulerTuning::from_runtime_pressure(frame_budget_ms, frame_ms).granularity_bias(),
            ));

            if resize_cooldown_frames > 0 {
                resize_cooldown_frames = resize_cooldown_frames.saturating_sub(1);
            }

            if frame_ms > frame_budget_ms * 1.15 {
                over_budget_streak = over_budget_streak.saturating_add(1);
                under_budget_streak = 0;
            } else if frame_ms < frame_budget_ms * 0.60 {
                under_budget_streak = under_budget_streak.saturating_add(1);
                over_budget_streak = 0;
            } else {
                over_budget_streak = 0;
                under_budget_streak = 0;
            }

            if resize_cooldown_frames == 0
                && over_budget_streak >= 3
                && internal_width > 160
                && internal_height > 90
            {
                internal_width = ((internal_width as f64) * 0.82).round() as usize;
                internal_height = ((internal_height as f64) * 0.82).round() as usize;
                internal_width = internal_width.max(160).min(output_width.max(160));
                internal_height = internal_height.max(90).min(output_height.max(90));
                renderer = Renderer::with_resolution(internal_width, internal_height);
                scheduler = TileScheduler::new_tuned(internal_width, internal_height, realtime_threads, scheduler_tuning);
                resize_cooldown_frames = 18;
                over_budget_streak = 0;
                under_budget_streak = 0;
            } else if resize_cooldown_frames == 0
                && under_budget_streak >= 8
                && internal_width < output_width
                && internal_height < output_height
            {
                internal_width = ((internal_width as f64) * 1.10).round() as usize;
                internal_height = ((internal_height as f64) * 1.10).round() as usize;
                internal_width = internal_width.max(160).min(output_width.max(160));
                internal_height = internal_height.max(90).min(output_height.max(90));
                renderer = Renderer::with_resolution(internal_width, internal_height);
                scheduler = TileScheduler::new_tuned(internal_width, internal_height, realtime_threads, scheduler_tuning);
                resize_cooldown_frames = 24;
                over_budget_streak = 0;
                under_budget_streak = 0;
            }

            if idx % 30 == 0 {
                let adaptation_state = RuntimeAdaptationState {
                    target_frame_ms: frame_budget_ms,
                    frame_p50_ms: frame_ms,
                    frame_p95_ms: frame_ms,
                    frame_p99_ms: frame_ms,
                    jitter_ms: 0.0,
                    quality_bias: 1.0,
                    sample_pressure_scale,
                    scheduler_granularity: scheduler_tuning.granularity_bias(),
                    substeps: 1,
                    internal_width,
                    internal_height,
                    output_width,
                    output_height,
                    resize_cooldown_frames,
                    over_budget_streak,
                    under_budget_streak,
                };
                crate::runtime_log!("animation adaptation {}", format_adaptation(&adaptation_state));
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

fn smooth_runtime_pressure(current: f64, target: f64) -> f64 {
    smooth_runtime_metric(current, target, 0.12, 0.30, 0.02)
}

fn smooth_runtime_granularity(current: f64, target: f64) -> f64 {
    smooth_runtime_metric(current, target, 0.14, 0.34, 0.03)
}

fn smooth_runtime_metric(current: f64, target: f64, rise_alpha: f64, fall_alpha: f64, dead_band: f64) -> f64 {
    let delta = target - current;
    if delta.abs() <= dead_band {
        return current;
    }
    let alpha = if delta > 0.0 { rise_alpha } else { fall_alpha };
    current + delta * alpha.clamp(0.0, 1.0)
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

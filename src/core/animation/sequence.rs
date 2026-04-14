use std::{error::Error, path::PathBuf};

use crate::api::scene_descriptor::SceneDescriptor;
use crate::core::engine::rendering::renderer::{Renderer, types::RenderPreset};
use crate::core::engine::rendering::raytracing::{Camera, Vec3};
use crate::core::engine::rendering::raytracing::acceleration::BvhNode;
use crate::core::scheduler::adaptive::TileScheduler;

use super::clip::AnimationClip;

#[derive(Debug, Clone)]
pub struct FrameResult {
    pub frame:       usize,
    pub time_secs:   f64,
    pub output_path: PathBuf,
    pub duration_ms: u128,
}

#[derive(Debug, Clone)]
pub struct SequenceResult {
    pub frames:      Vec<FrameResult>,
    pub total_ms:    u128,
    pub output_dir:  PathBuf,
    pub frame_count: usize,
    pub fps:         f64,
}

impl SequenceResult {
    pub fn average_frame_ms(&self) -> f64 {
        if self.frames.is_empty() {
            return 0.0;
        }
        self.total_ms as f64 / self.frames.len() as f64
    }
}

pub struct FrameSequencer {
    pub base:         SceneDescriptor,
    pub clip:         AnimationClip,
    pub output_dir:   PathBuf,
    pub frame_prefix: String,
    pub preset:       RenderPreset,
    pub width:        usize,
    pub height:       usize,
}

impl FrameSequencer {
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
        let scene_objects = base_scene.objects.len() + base_scene.triangles.len();
        let pixel_work = config.width * config.height * config.base_samples_per_pixel as usize;
        let bvh_factor = ((scene_objects.max(1) as f64).log2() / 10.0).clamp(0.3, 1.0);
        let work_estimate = pixel_work as f64 * bvh_factor;
        let max_threads = renderer.hw_caps.optimal_render_threads_for_input(pixel_work);
        let thread_count = (work_estimate / 10_000_000.0).ceil().clamp(1.0, max_threads as f64) as usize;
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

    pub fn render_all_to_window(&self) -> Result<SequenceResult, Box<dyn Error>> {
        use crate::core::engine::acces_hardware::{
            precise_timestamp_ns, elapsed_ms as hw_elapsed,
            NativeWindow, pixels_from_vec3,
        };

        let frame_count = self.clip.frame_count();
        let renderer = Renderer::with_resolution(self.width, self.height);
        let mut frames = Vec::with_capacity(frame_count);
        let t_total = precise_timestamp_ns();

        let aspect = self.width as f64 / self.height as f64;
        let (mut base_scene, _) = self.base.clone().into_builder().build(aspect);

        let t_bvh = precise_timestamp_ns();
        let bvh = BvhNode::build(&base_scene);
        let bvh_ms = hw_elapsed(t_bvh, precise_timestamp_ns());
        eprintln!("window: BVH cached in {:.2}ms for {} frames", bvh_ms, frame_count);

        let config = renderer.config_for(self.preset);
        let scene_objects = base_scene.objects.len() + base_scene.triangles.len();
        let pixel_work = config.width * config.height * config.base_samples_per_pixel as usize;
        let bvh_factor = ((scene_objects.max(1) as f64).log2() / 10.0).clamp(0.3, 1.0);
        let work_estimate = pixel_work as f64 * bvh_factor;
        let max_threads = renderer.hw_caps.optimal_render_threads_for_input(pixel_work);
        let thread_count = (work_estimate / 10_000_000.0).ceil().clamp(1.0, max_threads as f64) as usize;
        let scheduler = TileScheduler::new(self.width, self.height, thread_count);
        eprintln!(
            "window: scheduler cached — {} threads/{} for {} frames",
            thread_count, max_threads, frame_count,
        );

        let title = format!("EngineRenderer — {}x{} @ {}fps", self.width, self.height, self.clip.fps as u32);
        let mut window = NativeWindow::open(self.width, self.height, &title);
        if window.is_none() {
            eprintln!("window: display unavailable, falling back to disk render");
            return self.render_all();
        }
        let window = window.as_mut().unwrap();

        let frame_budget_ns = (1_000_000_000.0 / self.clip.fps) as u64;

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

            let argb = pixels_from_vec3(&color, self.width, self.height);
            window.present_frame(&argb, self.width, self.height);

            eprintln!("window: frame {}/{} t={:.3}s ({:.1}ms)",
                idx + 1, frame_count, time, frame_ms);

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

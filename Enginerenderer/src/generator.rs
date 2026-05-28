use std::path::PathBuf;

use enginerenderer::api::animation::AnimationClip;
use enginerenderer::api::engine::EngineApi;
use enginerenderer::api::scenes::SceneDescriptor;
use enginerenderer::api::types::core::{Quality, RenderRequest};

pub fn is_video_mode(args: &[String]) -> bool {
    args.iter().any(|arg| {
        arg == "video"
            || arg == "--video"
            || arg.starts_with("--duration")
            || arg.starts_with("--output-dir")
            || arg.starts_with("--output-mp4")
            || arg.starts_with("--prefix")
            || arg.starts_with("--scene")
            || arg.starts_with("--scene-file")
            || arg.starts_with("--quality")
    })
}

pub fn run(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let defaults = RenderRequest::preview();
    let duration = parse_u32_arg(args, "--duration", 5).max(1);
    let fps = parse_u32_arg(args, "--fps", 30).max(1);
    let width = parse_u32_arg(args, "--width", defaults.width as u32).max(1);
    let height = parse_u32_arg(args, "--height", defaults.height as u32).max(1);
    let quality = parse_quality(args);
    let output_dir = parse_path_arg(args, "--output-dir").unwrap_or_else(|| PathBuf::from("output/video"));
    let output_mp4 = parse_path_arg(args, "--output-mp4").unwrap_or_else(|| output_dir.join("animation.mp4"));
    let frame_prefix = parse_string_arg(args, "--prefix").unwrap_or_else(|| String::from("frame"));

    std::fs::create_dir_all(&output_dir)?;
    if let Some(parent) = output_mp4.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }

    let frame_output_dir = output_dir.join(".frames_tmp");
    if frame_output_dir.exists() {
        std::fs::remove_dir_all(&frame_output_dir)?;
    }
    std::fs::create_dir_all(&frame_output_dir)?;

    let request = RenderRequest::preview()
        .with_quality(quality)
        .with_resolution(width as usize, height as usize)
        .with_output(frame_output_dir.clone(), String::from("frame.ppm"));

    let clip = AnimationClip::new(duration as f64, fps as f64);
    let base = load_scene(args)?;
    let api = EngineApi::new();
    let sequence = api.render_animation(base, clip, &request, &frame_prefix)?;
    api.encode_animation_mp4(&sequence, &output_mp4)?;

    for frame in &sequence.frames {
        if frame.output_path.exists() {
            std::fs::remove_file(&frame.output_path)?;
        }
    }
    if frame_output_dir.exists() {
        std::fs::remove_dir_all(&frame_output_dir)?;
    }

    eprintln!("video generated: {}", output_mp4.display());
    Ok(())
}

fn load_scene(args: &[String]) -> Result<SceneDescriptor, Box<dyn std::error::Error>> {
    if let Some(path) = parse_path_arg(args, "--scene-file") {
        return Ok(SceneDescriptor::load_from_file(path)?);
    }
    Ok(SceneDescriptor::default())
}

fn parse_u32_arg(args: &[String], key: &str, default_value: u32) -> u32 {
    if let Some(value) = parse_string_arg(args, key)
        && let Ok(parsed) = value.parse::<u32>()
    {
        return parsed;
    }
    default_value
}

fn parse_string_arg(args: &[String], key: &str) -> Option<String> {
    for (index, arg) in args.iter().enumerate() {
        if let Some(value) = arg.strip_prefix(&(String::from(key) + "=")) {
            return Some(value.to_string());
        }
        if arg == key {
            return args.get(index + 1).cloned();
        }
    }
    None
}

fn parse_path_arg(args: &[String], key: &str) -> Option<PathBuf> {
    parse_string_arg(args, key).map(PathBuf::from)
}

fn parse_quality(args: &[String]) -> Quality {
    let Some(raw) = parse_string_arg(args, "--quality") else {
        return Quality::Preview;
    };

    match raw.trim().to_ascii_lowercase().as_str() {
        "preview" => Quality::Preview,
        "hd" => Quality::Hd,
        "production" => Quality::Production,
        _ => Quality::Preview,
    }
}

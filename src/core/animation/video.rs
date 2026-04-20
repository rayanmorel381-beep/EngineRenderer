use std::{
    error::Error,
    fmt,
    path::Path,
    process::Command,
};

use super::sequence::SequenceResult;

#[derive(Debug)]
pub struct FfmpegNotFound;

impl fmt::Display for FfmpegNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ffmpeg not found in PATH — install ffmpeg to encode video sequences")
    }
}

impl Error for FfmpegNotFound {}

pub struct VideoExporter;

impl VideoExporter {
    pub fn encode_h264<P: AsRef<Path>, Q: AsRef<Path>>(
        frame_dir:    P,
        frame_prefix: &str,
        fps:          f64,
        output_path:  Q,
    ) -> Result<(), Box<dyn Error>> {
        if !ffmpeg_available() {
            return Err(Box::new(FfmpegNotFound));
        }

        let frame_dir   = frame_dir.as_ref();
        let output_path = output_path.as_ref();

        if let Some(parent) = output_path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }

        let input_pattern = frame_dir.join(format!("{prefix}_%05d.png", prefix = frame_prefix));

        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-r",          &format!("{fps:.6}"),
                "-i",          input_pattern.to_str().unwrap_or(""),
                "-vcodec",     "libx264",
                "-crf",        "18",
                "-pix_fmt",    "yuv420p",
                "-movflags",   "+faststart",
                output_path.to_str().unwrap_or(""),
            ])
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(format!("ffmpeg exited with status {status}").into())
        }
    }

    pub fn encode_from_result<P: AsRef<Path>>(
        result:      &SequenceResult,
        output_path: P,
    ) -> Result<(), Box<dyn Error>> {
        if result.frames.is_empty() {
            return Err("no frames to encode".into());
        }
        let first    = &result.frames[0].output_path;
        let ext      = first.extension().and_then(|e| e.to_str()).unwrap_or("png");
        let stem     = first
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.rfind('_').map(|i| &n[..i]))
            .unwrap_or("frame");

        if ext != "png" {
            return Err(
                "video encoding requires PNG frames — rerun sequence with .png prefix".into()
            );
        }

        Self::encode_h264(&result.output_dir, stem, result.fps, output_path)
    }
}

fn ffmpeg_available() -> bool {
    Command::new("ffmpeg")
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

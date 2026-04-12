use std::{fs, path::Path};

use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone, Copy)]
pub struct TextureImageSummary {
    pub average_color: Vec3,
    pub detail: f64,
    pub width: u32,
    pub height: u32,
}

impl TextureImageSummary {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        let bytes = fs::read(path).ok()?;
        if bytes.starts_with(b"P6") || bytes.starts_with(b"P5") || bytes.starts_with(b"P3") {
            Self::from_pnm(&bytes)
        } else {
            Self::from_binary_fallback(&bytes)
        }
    }

    fn from_pnm(bytes: &[u8]) -> Option<Self> {
        let mut cursor = 0usize;
        let magic = Self::read_token(bytes, &mut cursor)?;
        let width = Self::read_token(bytes, &mut cursor)?.parse::<u32>().ok()?;
        let height = Self::read_token(bytes, &mut cursor)?.parse::<u32>().ok()?;
        let max_value = Self::read_token(bytes, &mut cursor)?
            .parse::<u32>()
            .ok()?
            .max(1);

        while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }

        match magic.as_str() {
            "P6" if max_value <= 255 => {
                Self::summarize_binary_rgb(&bytes[cursor..], width, height, max_value)
            }
            "P5" if max_value <= 255 => {
                Self::summarize_binary_gray(&bytes[cursor..], width, height, max_value)
            }
            "P3" => Self::summarize_ascii_rgb(bytes, cursor, width, height, max_value),
            _ => Self::from_binary_fallback(bytes),
        }
    }

    fn summarize_binary_rgb(
        data: &[u8],
        width: u32,
        height: u32,
        max_value: u32,
    ) -> Option<Self> {
        let pixel_count = width as usize * height as usize;
        if pixel_count == 0 || data.len() < 3 {
            return None;
        }

        let step = (pixel_count / 4096).max(1);
        let scale = max_value as f64;
        let mut sum = Vec3::ZERO;
        let mut count = 0.0_f64;
        let mut detail = 0.0_f64;
        let mut previous: Option<Vec3> = None;

        for index in (0..pixel_count).step_by(step) {
            let base = index * 3;
            if base + 2 >= data.len() {
                break;
            }

            let color = Vec3::new(
                data[base] as f64 / scale,
                data[base + 1] as f64 / scale,
                data[base + 2] as f64 / scale,
            );
            if let Some(last) = previous {
                detail += (color - last).length();
            }
            previous = Some(color);
            sum += color;
            count += 1.0;
        }

        Self::finalize(sum, count, detail, width, height)
    }

    fn summarize_binary_gray(
        data: &[u8],
        width: u32,
        height: u32,
        max_value: u32,
    ) -> Option<Self> {
        let pixel_count = width as usize * height as usize;
        if pixel_count == 0 || data.is_empty() {
            return None;
        }

        let step = (pixel_count / 4096).max(1);
        let scale = max_value as f64;
        let mut sum = Vec3::ZERO;
        let mut count = 0.0_f64;
        let mut detail = 0.0_f64;
        let mut previous: Option<Vec3> = None;

        for index in (0..pixel_count).step_by(step) {
            if index >= data.len() {
                break;
            }

            let luminance = data[index] as f64 / scale;
            let color = Vec3::splat(luminance);
            if let Some(last) = previous {
                detail += (color - last).length();
            }
            previous = Some(color);
            sum += color;
            count += 1.0;
        }

        Self::finalize(sum, count, detail, width, height)
    }

    fn summarize_ascii_rgb(
        bytes: &[u8],
        mut cursor: usize,
        width: u32,
        height: u32,
        max_value: u32,
    ) -> Option<Self> {
        let pixel_count = width as usize * height as usize;
        if pixel_count == 0 {
            return None;
        }

        let step = (pixel_count / 4096).max(1);
        let scale = max_value as f64;
        let mut sum = Vec3::ZERO;
        let mut count = 0.0_f64;
        let mut detail = 0.0_f64;
        let mut previous: Option<Vec3> = None;

        for pixel_index in 0..pixel_count {
            let r = Self::read_token(bytes, &mut cursor)?
                .parse::<u32>()
                .ok()? as f64
                / scale;
            let g = Self::read_token(bytes, &mut cursor)?
                .parse::<u32>()
                .ok()? as f64
                / scale;
            let b = Self::read_token(bytes, &mut cursor)?
                .parse::<u32>()
                .ok()? as f64
                / scale;

            if pixel_index % step != 0 {
                continue;
            }

            let color = Vec3::new(r, g, b);
            if let Some(last) = previous {
                detail += (color - last).length();
            }
            previous = Some(color);
            sum += color;
            count += 1.0;
        }

        Self::finalize(sum, count, detail, width, height)
    }

    fn from_binary_fallback(bytes: &[u8]) -> Option<Self> {
        if bytes.is_empty() {
            return None;
        }

        let mut sum = Vec3::ZERO;
        let mut count = 0.0_f64;
        let mut detail = 0.0_f64;
        let mut previous: Option<Vec3> = None;

        for chunk in bytes.chunks(3).take(4096) {
            let color = Vec3::new(
                *chunk.first().unwrap_or(&0) as f64 / 255.0,
                *chunk.get(1).unwrap_or(&0) as f64 / 255.0,
                *chunk.get(2).unwrap_or(&0) as f64 / 255.0,
            );
            if let Some(last) = previous {
                detail += (color - last).length();
            }
            previous = Some(color);
            sum += color;
            count += 1.0;
        }

        let width = (count.sqrt().round() as u32).max(1);
        let height = ((count as u32).saturating_add(width - 1) / width).max(1);
        Self::finalize(sum, count, detail, width, height)
    }

    fn finalize(sum: Vec3, count: f64, detail: f64, width: u32, height: u32) -> Option<Self> {
        if count <= f64::EPSILON {
            None
        } else {
            Some(Self {
                average_color: (sum / count).clamp(0.0, 1.0),
                detail: (detail / count).clamp(0.0, 1.0),
                width,
                height,
            })
        }
    }

    fn read_token(bytes: &[u8], cursor: &mut usize) -> Option<String> {
        while *cursor < bytes.len() {
            let byte = bytes[*cursor];
            if byte == b'#' {
                while *cursor < bytes.len() && bytes[*cursor] != b'\n' {
                    *cursor += 1;
                }
            } else if byte.is_ascii_whitespace() {
                *cursor += 1;
            } else {
                break;
            }
        }

        if *cursor >= bytes.len() {
            return None;
        }

        let start = *cursor;
        while *cursor < bytes.len() && !bytes[*cursor].is_ascii_whitespace() {
            *cursor += 1;
        }

        std::str::from_utf8(&bytes[start..*cursor])
            .ok()
            .map(str::to_string)
    }
}

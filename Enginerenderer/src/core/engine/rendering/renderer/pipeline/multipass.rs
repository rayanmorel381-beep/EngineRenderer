use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::rendering::framebuffer::FrameBuffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassKind {
    Depth,
    GBuffer,
    Lighting,
    AmbientOcclusion,
    Reflection,
    VolumetricFog,
    Bloom,
    ToneMapping,
    Taa,
    Present,
}

impl PassKind {
    pub fn label(self) -> &'static str {
        match self {
            PassKind::Depth => "Depth Pre-Pass",
            PassKind::GBuffer => "G-Buffer",
            PassKind::Lighting => "Lighting",
            PassKind::AmbientOcclusion => "SSAO",
            PassKind::Reflection => "SSR",
            PassKind::VolumetricFog => "Volumetric Fog",
            PassKind::Bloom => "Bloom",
            PassKind::ToneMapping => "Tone Mapping",
            PassKind::Taa => "TAA",
            PassKind::Present => "Present",
        }
    }
}

#[derive(Debug, Clone)]
pub struct PassStats {
    pub kind: PassKind,
    pub duration_ns: u64,
    pub pixels_written: usize,
}

#[derive(Debug, Clone)]
pub struct MultiPassPipeline {
    pub passes: Vec<PassKind>,
    pub enabled_passes: Vec<bool>,
    pub pass_stats: Vec<PassStats>,
}

impl Default for MultiPassPipeline {
    fn default() -> Self {
        let passes = vec![
            PassKind::Depth,
            PassKind::GBuffer,
            PassKind::AmbientOcclusion,
            PassKind::Lighting,
            PassKind::Reflection,
            PassKind::VolumetricFog,
            PassKind::Bloom,
            PassKind::ToneMapping,
            PassKind::Taa,
            PassKind::Present,
        ];
        let n = passes.len();
        Self {
            passes,
            enabled_passes: vec![true; n],
            pass_stats: Vec::new(),
        }
    }
}

impl MultiPassPipeline {
    pub fn with_pass_enabled(mut self, kind: PassKind, enabled: bool) -> Self {
        for (i, pass) in self.passes.iter().enumerate() {
            if *pass == kind { self.enabled_passes[i] = enabled; }
        }
        self
    }

    pub fn preview() -> Self {
        Self::default()
            .with_pass_enabled(PassKind::AmbientOcclusion, false)
            .with_pass_enabled(PassKind::Reflection, false)
            .with_pass_enabled(PassKind::VolumetricFog, false)
    }

    pub fn production() -> Self {
        Self::default()
    }

    pub fn execute_passes(
        &mut self,
        framebuffer: &mut FrameBuffer,
        start_ns: u64,
    ) {
        self.pass_stats.clear();
        let width = framebuffer.width;
        let height = framebuffer.height;

        for (i, &kind) in self.passes.iter().enumerate() {
            if !self.enabled_passes[i] { continue; }
            let t0 = start_ns + i as u64 * 1000;
            let pixels_written = match kind {
                PassKind::Depth => {
                    for d in &mut framebuffer.depth { *d = d.min(1e6); }
                    width * height
                }
                PassKind::GBuffer => width * height,
                PassKind::AmbientOcclusion => {
                    apply_ssao(framebuffer);
                    width * height
                }
                PassKind::Reflection => {
                    apply_ssr(framebuffer);
                    width * height
                }
                PassKind::VolumetricFog => {
                    apply_volumetric_fog(framebuffer);
                    width * height
                }
                PassKind::Bloom => {
                    width * height
                }
                PassKind::ToneMapping => {
                    apply_aces(framebuffer);
                    width * height
                }
                PassKind::Taa => width * height,
                PassKind::Lighting | PassKind::Present => width * height,
            };
            self.pass_stats.push(PassStats { kind, duration_ns: t0, pixels_written });
        }
    }

    pub fn total_duration_ns(&self) -> u64 {
        self.pass_stats.iter().map(|s| s.duration_ns).sum()
    }

    pub fn is_pass_enabled(&self, kind: PassKind) -> bool {
        self.passes.iter().zip(self.enabled_passes.iter())
            .any(|(p, &e)| *p == kind && e)
    }
}

fn apply_ssao(fb: &mut FrameBuffer) {
    let w = fb.width;
    let h = fb.height;
    for y in 1..(h.saturating_sub(1)) {
        for x in 1..(w.saturating_sub(1)) {
            let idx = y * w + x;
            let d = fb.depth[idx];
            if !d.is_finite() || d > 1e5 { continue; }
            let occlusion = sample_neighborhood_occlusion(&fb.depth, x, y, w, h);
            fb.color[idx] = fb.color[idx] * (1.0 - occlusion * 0.35);
        }
    }
}

fn sample_neighborhood_occlusion(depth: &[f64], x: usize, y: usize, w: usize, h: usize) -> f64 {
    let center_d = depth[y * w + x];
    if !center_d.is_finite() { return 0.0; }
    let mut occlusion = 0.0_f64;
    let mut count = 0usize;
    for dy in 0..3usize {
        for dx in 0..3usize {
            let nx = x + dx;
            let ny = y + dy;
            if nx >= 2 && ny >= 2 && (nx - 2) < w && (ny - 2) < h {
                let nd = depth[(ny - 1) * w + (nx - 1)];
                if nd.is_finite() && nd < center_d - 0.01 {
                    occlusion += (center_d - nd).min(1.0);
                }
                count += 1;
            }
        }
    }
    if count > 0 { (occlusion / count as f64).min(1.0) } else { 0.0 }
}

fn apply_ssr(fb: &mut FrameBuffer) {
    let w = fb.width;
    let h = fb.height;
    let snapshot = fb.color.clone();
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let pixel = snapshot[idx];
            let lum = pixel.x * 0.299 + pixel.y * 0.587 + pixel.z * 0.114;
            if lum < 0.6 { continue; }
            let rx = (x + 3).min(w - 1);
            let ry = (y + 2).min(h - 1);
            let reflect_sample = snapshot[ry * w + rx];
            fb.color[idx] += reflect_sample * 0.04;
        }
    }
}

fn apply_volumetric_fog(fb: &mut FrameBuffer) {
    let fog_color = Vec3::new(0.7, 0.75, 0.85);
    for (i, color) in fb.color.iter_mut().enumerate() {
        let d = fb.depth[i];
        if !d.is_finite() { continue; }
        let fog_density = 0.015_f64;
        let transmittance = (-fog_density * d.min(200.0)).exp();
        *color = *color * transmittance + fog_color * (1.0 - transmittance) * 0.8;
    }
}

fn apply_aces(fb: &mut FrameBuffer) {
    for pixel in &mut fb.color {
        *pixel = aces_filmic(*pixel);
    }
}

#[inline]
fn aces_filmic(color: Vec3) -> Vec3 {
    let a = 2.51_f64;
    let b = 0.03_f64;
    let c = 2.43_f64;
    let d = 0.59_f64;
    let e = 0.14_f64;
    let mapped = |x: f64| ((x * (a * x + b)) / (x * (c * x + d) + e)).clamp(0.0, 1.0);
    Vec3::new(mapped(color.x), mapped(color.y), mapped(color.z))
}

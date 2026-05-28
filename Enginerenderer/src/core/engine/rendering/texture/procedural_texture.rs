use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone, Copy)]
pub enum TextureKind {
    BrushedMetal,
    RockyMineral,
    FrozenCrystal,
    OceanicBands,
}

#[derive(Debug, Clone, Copy)]
pub struct ProceduralTexture {
    pub base_color: Vec3,
    pub accent_color: Vec3,
    pub scale: f64,
    pub detail_boost: f64,
    pub kind: TextureKind,
}

impl ProceduralTexture {
    pub fn brushed_space_metal() -> Self {
        Self {
            base_color: Vec3::new(0.42, 0.46, 0.52),
            accent_color: Vec3::new(0.78, 0.82, 0.88),
            scale: 8.0,
            detail_boost: 0.22,
            kind: TextureKind::BrushedMetal,
        }
    }

    pub fn rocky_planet(base_color: Vec3) -> Self {
        Self {
            base_color,
            accent_color: base_color.lerp(Vec3::new(0.82, 0.76, 0.68), 0.35),
            scale: 5.5,
            detail_boost: 0.28,
            kind: TextureKind::RockyMineral,
        }
    }

    pub fn frozen_crystal() -> Self {
        Self {
            base_color: Vec3::new(0.72, 0.86, 0.96),
            accent_color: Vec3::new(0.94, 0.97, 1.0),
            scale: 10.5,
            detail_boost: 0.18,
            kind: TextureKind::FrozenCrystal,
        }
    }

    pub fn oceanic_surface() -> Self {
        Self {
            base_color: Vec3::new(0.10, 0.28, 0.62),
            accent_color: Vec3::new(0.42, 0.78, 0.96),
            scale: 6.8,
            detail_boost: 0.16,
            kind: TextureKind::OceanicBands,
        }
    }

    pub fn sample(&self, point: Vec3) -> Vec3 {
        self.sample_uv(point, None, 1.0)
    }

    pub fn sample_uv(&self, point: Vec3, uv: Option<(f64, f64)>, uv_scale: f64) -> Vec3 {
        let (u, v) = uv.unwrap_or((point.x.abs().fract(), point.z.abs().fract()));
        let uv_wave = ((u * uv_scale * self.scale * std::f64::consts::TAU).sin() * 0.5 + 0.5)
            * 0.58
            + ((v * uv_scale * self.scale * std::f64::consts::PI).cos() * 0.5 + 0.5) * 0.42;
        let primary = ((point.x + point.z * 0.35) * self.scale).sin() * 0.5 + 0.5;
        let secondary =
            ((point.y * 0.55 - point.z * 0.25) * self.scale * 1.37).cos() * 0.5 + 0.5;
        let blend = (primary * 0.42 + secondary * 0.22 + uv_wave * 0.36).clamp(0.0, 1.0);

        match self.kind {
            TextureKind::BrushedMetal => self.base_color.lerp(self.accent_color, blend.powf(1.6)),
            TextureKind::RockyMineral => {
                self.base_color.lerp(self.accent_color, blend.powf(0.85))
            }
            TextureKind::FrozenCrystal => self.base_color.lerp(self.accent_color, blend.powf(2.2)),
            TextureKind::OceanicBands => self.base_color.lerp(self.accent_color, blend.powf(1.2)),
        }
    }

    pub fn sample_normal(&self, point: Vec3) -> Vec3 {
        self.sample_normal_uv(point, None, 1.0)
    }

    pub fn sample_normal_uv(&self, point: Vec3, uv: Option<(f64, f64)>, uv_scale: f64) -> Vec3 {
        let (u, v) = uv.unwrap_or((point.x.abs().fract(), point.z.abs().fract()));
        let uv_phase = uv_scale * self.scale;
        let wave_x = ((point.x + point.y * 0.2 + u * 2.0) * self.scale * 1.15).sin()
            * self.detail_boost;
        let wave_y = ((point.y - point.z * 0.3 + v * 1.7) * self.scale * 0.92).cos()
            * self.detail_boost
            * 0.6;
        let wave_z = ((point.z + point.x * 0.4 + (u - v) * 1.4) * uv_phase * 1.31).sin()
            * self.detail_boost;
        Vec3::new(wave_x, wave_y, wave_z)
    }

    pub fn sample_roughness(&self, point: Vec3) -> f64 {
        self.sample_roughness_uv(point, None, 1.0)
    }

    pub fn sample_roughness_uv(&self, point: Vec3, uv: Option<(f64, f64)>, uv_scale: f64) -> f64 {
        let (u, v) = uv.unwrap_or((point.x.abs().fract(), point.z.abs().fract()));
        let micro = ((point.x - point.z + (u - v) * uv_scale * 2.0) * self.scale * 0.75).sin()
            * 0.5
            + 0.5;
        (0.18 + micro * 0.62).clamp(0.0, 1.0)
    }
}

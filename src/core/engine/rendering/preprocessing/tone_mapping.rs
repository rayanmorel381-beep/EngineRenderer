use crate::core::engine::rendering::raytracing::Vec3;

/// Tone mapping operator used to map HDR values to display range.
#[derive(Debug, Clone, Copy)]
pub enum ToneMappingOperator {
    /// ACES fitted curve.
    Aces,
    /// Reinhard global tone mapping.
    Reinhard,
    /// Filmic curve.
    Filmic,
    /// AgX-like curve.
    AgX,
}

impl ToneMappingOperator {
    /// Applies tone mapping to an HDR color with exposure.
    pub fn apply(self, color: Vec3, exposure: f64) -> Vec3 {
        let exposed = color * exposure.max(0.1);
        match self {
            Self::Aces => Vec3::new(
                aces_curve(exposed.x),
                aces_curve(exposed.y),
                aces_curve(exposed.z),
            ),
            Self::Reinhard => {
                let luma = 0.2126 * exposed.x + 0.7152 * exposed.y + 0.0722 * exposed.z;
                let mapped_luma = luma / (1.0 + luma);
                let scale = if luma > f64::EPSILON {
                    mapped_luma / luma
                } else {
                    1.0
                };
                (exposed * scale).clamp(0.0, 1.0)
            }
            Self::Filmic => Vec3::new(
                filmic_curve(exposed.x),
                filmic_curve(exposed.y),
                filmic_curve(exposed.z),
            ),
            Self::AgX => Vec3::new(
                agx_curve(exposed.x),
                agx_curve(exposed.y),
                agx_curve(exposed.z),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorGrading {
    pub lift: Vec3,
    pub gamma: Vec3,
    pub gain: Vec3,
    pub saturation: f64,
    pub contrast: f64,
    pub temperature: f64,
}

impl Default for ColorGrading {
    fn default() -> Self {
        Self {
            lift: Vec3::ZERO,
            gamma: Vec3::ONE,
            gain: Vec3::ONE,
            saturation: 1.0,
            contrast: 1.0,
            temperature: 0.0,
        }
    }
}

impl ColorGrading {
    pub fn cinematic() -> Self {
        Self {
            lift: Vec3::new(0.002, 0.001, 0.003),
            gamma: Vec3::new(1.02, 1.0, 0.98),
            gain: Vec3::new(1.02, 1.0, 0.98),
            saturation: 1.08,
            contrast: 1.05,
            temperature: -0.02,
        }
    }

    pub fn apply(&self, color: Vec3) -> Vec3 {
        let lifted = color + self.lift;
        let gained = lifted * self.gain;
        let gamma_corrected = Vec3::new(
            gained.x.max(0.0).powf(1.0 / self.gamma.x.max(0.01)),
            gained.y.max(0.0).powf(1.0 / self.gamma.y.max(0.01)),
            gained.z.max(0.0).powf(1.0 / self.gamma.z.max(0.01)),
        );

        let luma = 0.2126 * gamma_corrected.x + 0.7152 * gamma_corrected.y + 0.0722 * gamma_corrected.z;
        let gray = Vec3::splat(luma);
        let saturated = gray.lerp(gamma_corrected, self.saturation);

        let contrasted = (saturated - Vec3::splat(0.5)) * self.contrast + Vec3::splat(0.5);

        let temp_shift = if self.temperature > 0.0 {
            Vec3::new(self.temperature * 0.1, 0.0, -self.temperature * 0.06)
        } else {
            Vec3::new(self.temperature * 0.06, 0.0, -self.temperature * 0.1)
        };

        (contrasted + temp_shift).clamp(0.0, 1.5)
    }
}

fn aces_curve(x: f64) -> f64 {
    ((x * (2.51 * x + 0.03)) / (x * (2.43 * x + 0.59) + 0.14)).clamp(0.0, 1.0)
}

fn filmic_curve(x: f64) -> f64 {
    let a = 0.22;
    let b = 0.30;
    let c = 0.10;
    let d = 0.20;
    let e = 0.01;
    let f = 0.30;
    let num = (x * (a * x + c * b) + d * e) / (x * (a * x + b) + d * f) - e / f;
    let white = 11.2;
    let denom = (white * (a * white + c * b) + d * e) / (white * (a * white + b) + d * f) - e / f;
    (num / denom.max(f64::EPSILON)).clamp(0.0, 1.0)
}

fn agx_curve(x: f64) -> f64 {
    let x = x.max(0.0);
    let compressed = x / (x + 0.66);
    let shaped = compressed * compressed * (3.0 - 2.0 * compressed);
    shaped.clamp(0.0, 1.0)
}

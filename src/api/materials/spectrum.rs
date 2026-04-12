use crate::core::engine::rendering::raytracing::Vec3;

/// Number of spectral bands (covers ~380 nm – 780 nm in equal steps).
pub const SPECTRAL_BANDS: usize = 16;

/// A sampled light spectrum.
///
/// Each band covers ~25 nm of the visible range.  Conversions to/from
/// sRGB are provided but the raw spectrum is always available for
/// physically accurate wavelength-dependent effects (dispersion,
/// chromatic aberration, thin-film interference, Rayleigh/Mie
/// scattering, Doppler shifting, etc.).
#[derive(Debug, Clone, Copy)]
pub struct Spectrum {
    /// Energy per band.  Index 0 ≈ 380 nm, index 15 ≈ 755 nm.
    pub bands: [f64; SPECTRAL_BANDS],
}

impl Spectrum {
    pub const ZERO: Self = Self {
        bands: [0.0; SPECTRAL_BANDS],
    };

    /// Flat (white) spectrum with a given power level per band.
    pub fn flat(power: f64) -> Self {
        Self {
            bands: [power; SPECTRAL_BANDS],
        }
    }

    /// Build a spectrum from a single dominant wavelength (nm) and power.
    /// The energy is spread with a Gaussian falloff around `wavelength_nm`.
    pub fn from_wavelength(wavelength_nm: f64, power: f64, spread_nm: f64) -> Self {
        let mut bands = [0.0; SPECTRAL_BANDS];
        let step = (780.0 - 380.0) / SPECTRAL_BANDS as f64;
        for (i, band) in bands.iter_mut().enumerate() {
            let center = 380.0 + step * (i as f64 + 0.5);
            let delta = (center - wavelength_nm) / spread_nm.max(1.0);
            *band = power * (-0.5 * delta * delta).exp();
        }
        Self { bands }
    }

    /// Black-body spectrum at a given temperature (Kelvin).
    pub fn black_body(temperature_k: f64, peak_power: f64) -> Self {
        let mut bands = [0.0; SPECTRAL_BANDS];
        let step = (780.0 - 380.0) / SPECTRAL_BANDS as f64;
        let mut max_val: f64 = 0.0;
        for (i, band) in bands.iter_mut().enumerate() {
            let lambda_nm = 380.0 + step * (i as f64 + 0.5);
            let lambda_um = lambda_nm * 1e-3;
            let x = 14388.0 / (lambda_um * temperature_k);
            let radiance = 1.0 / (lambda_um.powi(5) * (x.exp() - 1.0).max(1e-30));
            *band = radiance;
            max_val = max_val.max(radiance);
        }
        if max_val > 0.0 {
            for b in &mut bands {
                *b = *b / max_val * peak_power;
            }
        }
        Self { bands }
    }

    /// Convert to approximate linear RGB (CIE 1931 2° observer, D65).
    pub fn to_rgb(&self) -> Vec3 {
        #[allow(clippy::excessive_precision)]
        const XYZ_TABLE: [[f64; 3]; SPECTRAL_BANDS] = [
            [0.0143, 0.0004, 0.0679],
            [0.1344, 0.0040, 0.6456],
            [0.2839, 0.0116, 1.3856],
            [0.3285, 0.0230, 1.6230],
            [0.0956, 0.0600, 0.8130],
            [0.0096, 0.1390, 0.2720],
            [0.0633, 0.3230, 0.0782],
            [0.2074, 0.5030, 0.0203],
            [0.4412, 0.7100, 0.0039],
            [0.7010, 0.8620, 0.0002],
            [0.9763, 0.9540, 0.0000],
            [1.0263, 0.8540, 0.0000],
            [0.7570, 0.6420, 0.0000],
            [0.4257, 0.3810, 0.0000],
            [0.1582, 0.1750, 0.0000],
            [0.0452, 0.0540, 0.0000],
        ];

        let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
        for (i, &power) in self.bands.iter().enumerate() {
            x += power * XYZ_TABLE[i][0];
            y += power * XYZ_TABLE[i][1];
            z += power * XYZ_TABLE[i][2];
        }

        let r = 3.2406 * x - 1.5372 * y - 0.4986 * z;
        let g = -0.9689 * x + 1.8758 * y + 0.0415 * z;
        let b = 0.0557 * x - 0.2040 * y + 1.0570 * z;
        Vec3::new(r.max(0.0), g.max(0.0), b.max(0.0))
    }

    /// Convert to RGB with a caller-supplied 3×3 colour-space matrix.
    pub fn to_rgb_custom(&self, xyz_to_rgb: [[f64; 3]; 3]) -> Vec3 {
        #[allow(clippy::excessive_precision)]
        const XYZ_TABLE: [[f64; 3]; SPECTRAL_BANDS] = [
            [0.0143, 0.0004, 0.0679],
            [0.1344, 0.0040, 0.6456],
            [0.2839, 0.0116, 1.3856],
            [0.3285, 0.0230, 1.6230],
            [0.0956, 0.0600, 0.8130],
            [0.0096, 0.1390, 0.2720],
            [0.0633, 0.3230, 0.0782],
            [0.2074, 0.5030, 0.0203],
            [0.4412, 0.7100, 0.0039],
            [0.7010, 0.8620, 0.0002],
            [0.9763, 0.9540, 0.0000],
            [1.0263, 0.8540, 0.0000],
            [0.7570, 0.6420, 0.0000],
            [0.4257, 0.3810, 0.0000],
            [0.1582, 0.1750, 0.0000],
            [0.0452, 0.0540, 0.0000],
        ];

        let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
        for (i, &power) in self.bands.iter().enumerate() {
            x += power * XYZ_TABLE[i][0];
            y += power * XYZ_TABLE[i][1];
            z += power * XYZ_TABLE[i][2];
        }

        let m = xyz_to_rgb;
        Vec3::new(
            (m[0][0] * x + m[0][1] * y + m[0][2] * z).max(0.0),
            (m[1][0] * x + m[1][1] * y + m[1][2] * z).max(0.0),
            (m[2][0] * x + m[2][1] * y + m[2][2] * z).max(0.0),
        )
    }

    /// Approximate sRGB → flat spectrum (uniform-energy inverse).
    pub fn from_rgb(rgb: Vec3) -> Self {
        let mut bands = [0.0; SPECTRAL_BANDS];
        for (i, b) in bands.iter_mut().enumerate() {
            let t = i as f64 / (SPECTRAL_BANDS - 1) as f64;
            *b = rgb.z * (1.0 - t) + rgb.x * t + rgb.y * 0.5;
            *b /= 1.5;
        }
        Self { bands }
    }
}

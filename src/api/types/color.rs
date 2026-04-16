use crate::api::materials::Spectrum;

/// Linear RGB colour (0..∞ HDR).
#[derive(Debug, Clone, Copy)]
pub struct Color {
    /// Canal rouge linéaire.
    pub r: f64,
    /// Canal vert linéaire.
    pub g: f64,
    /// Canal bleu linéaire.
    pub b: f64,
}

impl Color {
    /// Noir linéaire absolu.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    /// Blanc linéaire unitaire.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    /// Construit une couleur linéaire.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// From 8-bit sRGB (0–255) → linear.
    pub fn from_srgb8(r: u8, g: u8, b: u8) -> Self {
        fn to_linear(c: u8) -> f64 {
            let s = c as f64 / 255.0;
            if s <= 0.04045 {
                s / 12.92
            } else {
                ((s + 0.055) / 1.055).powf(2.4)
            }
        }
        Self {
            r: to_linear(r),
            g: to_linear(g),
            b: to_linear(b),
        }
    }

    /// From hex string (#RRGGBB or RRGGBB).
    pub fn from_hex(hex: &str) -> Self {
        let h = hex.trim_start_matches('#');
        if h.len() < 6 {
            return Self::BLACK;
        }
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
        Self::from_srgb8(r, g, b)
    }

    /// From a colour temperature (Kelvin) → approximate linear RGB.
    pub fn from_temperature(kelvin: f64) -> Self {
        let spec = Spectrum::black_body(kelvin, 1.0);
        let rgb = spec.to_rgb();
        Self {
            r: rgb.x,
            g: rgb.y,
            b: rgb.z,
        }
    }

    /// To array `[r, g, b]` (API-friendly).
    pub fn to_array(self) -> [f64; 3] {
        [self.r, self.g, self.b]
    }
}

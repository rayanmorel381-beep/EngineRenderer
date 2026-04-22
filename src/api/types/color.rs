use crate::api::materials::Spectrum;

/// Linear RGB color used by the public API.
#[derive(Debug, Clone, Copy)]
pub struct Color {
    /// Red channel.
    pub r: f64,
    /// Green channel.
    pub g: f64,
    /// Blue channel.
    pub b: f64,
}

impl Color {
    /// Black color constant.
    pub const BLACK: Self = Self {
        r: 0.0,
        g: 0.0,
        b: 0.0,
    };
    /// White color constant.
    pub const WHITE: Self = Self {
        r: 1.0,
        g: 1.0,
        b: 1.0,
    };

    /// Creates a linear RGB color from channel values.
    pub const fn new(r: f64, g: f64, b: f64) -> Self {
        Self { r, g, b }
    }

    /// Converts 8-bit sRGB channels to linear RGB.
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

    /// Parses a hex color string and converts it to linear RGB.
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

    /// Approximates color from black-body temperature in kelvin.
    pub fn from_temperature(kelvin: f64) -> Self {
        let spec = Spectrum::black_body(kelvin, 1.0);
        let rgb = spec.to_rgb();
        Self {
            r: rgb.x,
            g: rgb.y,
            b: rgb.z,
        }
    }

    /// Returns the color as an RGB array.
    pub fn to_array(self) -> [f64; 3] {
        [self.r, self.g, self.b]
    }
}

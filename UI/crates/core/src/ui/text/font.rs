use std::sync::OnceLock;

use fontdue::{Font as FontdueFont, FontSettings};

const REGULAR_BYTES: &[u8] = include_bytes!("../../../assets/fonts/Inter-Regular.ttf");
const MEDIUM_BYTES: &[u8] = include_bytes!("../../../assets/fonts/Inter-Medium.ttf");
const ICONS_BYTES: &[u8] = include_bytes!("../../../assets/fonts/Lucide.ttf");

pub const RASTER_SIZE: f64 = 32.0;
pub const ICON_PUA_START: u32 = 0xE000;
pub const ICON_PUA_END: u32 = 0xF8FF;

static REGULAR: OnceLock<FontdueFont> = OnceLock::new();
static MEDIUM: OnceLock<FontdueFont> = OnceLock::new();
static ICONS: OnceLock<FontdueFont> = OnceLock::new();

pub fn regular() -> &'static FontdueFont {
    REGULAR.get_or_init(|| {
        FontdueFont::from_bytes(REGULAR_BYTES, FontSettings::default())
            .expect("Inter-Regular.ttf must be valid")
    })
}

pub fn medium() -> &'static FontdueFont {
    MEDIUM.get_or_init(|| {
        FontdueFont::from_bytes(MEDIUM_BYTES, FontSettings::default())
            .expect("Inter-Medium.ttf must be valid")
    })
}

pub fn icons() -> &'static FontdueFont {
    ICONS.get_or_init(|| {
        FontdueFont::from_bytes(ICONS_BYTES, FontSettings::default())
            .expect("Lucide.ttf must be valid")
    })
}

pub fn font_for_codepoint(codepoint: u32) -> &'static FontdueFont {
    if (ICON_PUA_START..=ICON_PUA_END).contains(&codepoint) {
        icons()
    } else {
        regular()
    }
}

pub struct Font {
    pub name: &'static str,
}

impl Font {
    pub const REGULAR: Self = Self { name: "Inter Regular" };
    pub const MEDIUM: Self = Self { name: "Inter Medium" };
    pub const ICONS: Self = Self { name: "Lucide" };

    pub fn fontdue(&self) -> &'static FontdueFont {
        match self.name {
            "Inter Medium" => medium(),
            "Lucide" => icons(),
            _ => regular(),
        }
    }

    pub fn line_height(&self, size: f64) -> f64 {
        let metrics = self.fontdue().horizontal_line_metrics(size as f32);
        metrics
            .map(|m| (m.ascent - m.descent + m.line_gap) as f64)
            .unwrap_or(size * 1.25)
    }

    pub fn ascent(&self, size: f64) -> f64 {
        self.fontdue()
            .horizontal_line_metrics(size as f32)
            .map(|m| m.ascent as f64)
            .unwrap_or(size * 0.8)
    }

    pub fn measure_line(&self, text: &str, size: f64) -> f64 {
        let mut width = 0.0_f64;
        let mut prev: Option<(char, &'static FontdueFont)> = None;
        for ch in text.chars() {
            let f = font_for_codepoint(ch as u32);
            if let Some((p, prev_font)) = prev
                && std::ptr::eq(prev_font, f)
            {
                width += f.horizontal_kern(p, ch, size as f32).unwrap_or(0.0) as f64;
            }
            let m = f.metrics(ch, size as f32);
            width += m.advance_width as f64;
            prev = Some((ch, f));
        }
        width
    }

    pub fn measure(&self, text: &str, size: f64) -> (f64, f64) {
        let lh = self.line_height(size);
        let lines = text.lines().count().max(1);
        let widest = text
            .lines()
            .map(|l| self.measure_line(l, size))
            .fold(0.0_f64, f64::max);
        (widest, lh * lines as f64)
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::REGULAR
    }
}

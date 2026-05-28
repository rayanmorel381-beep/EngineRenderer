use std::collections::HashMap;

use crate::ui::text::font::{self, RASTER_SIZE};

pub const ATLAS_WIDTH: u32 = 1024;
pub const ATLAS_HEIGHT: u32 = 1024;
const PADDING: u32 = 1;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Glyph {
    pub codepoint: u32,
    pub atlas_x: u32,
    pub atlas_y: u32,
    pub width: u32,
    pub height: u32,
    pub xmin: f64,
    pub ymin: f64,
    pub advance: f64,
}

pub struct GlyphAtlas {
    pub texture_width: u32,
    pub texture_height: u32,
    pub pixels: Vec<u8>,
    pub dirty: bool,
    glyphs: HashMap<u32, Glyph>,
    pen_x: u32,
    pen_y: u32,
    row_height: u32,
}

impl GlyphAtlas {
    pub fn empty() -> Self {
        Self {
            texture_width: ATLAS_WIDTH,
            texture_height: ATLAS_HEIGHT,
            pixels: vec![0; (ATLAS_WIDTH * ATLAS_HEIGHT) as usize],
            dirty: true,
            glyphs: HashMap::new(),
            pen_x: PADDING,
            pen_y: PADDING,
            row_height: 0,
        }
    }

    pub fn glyph(&self, codepoint: u32) -> Option<&Glyph> {
        self.glyphs.get(&codepoint)
    }

    pub fn ensure(&mut self, codepoint: u32) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&codepoint) {
            return Some(*g);
        }
        let ch = char::from_u32(codepoint)?;
        let f = font::font_for_codepoint(codepoint);
        let (metrics, bitmap) = f.rasterize(ch, RASTER_SIZE as f32);
        let w = metrics.width as u32;
        let h = metrics.height as u32;

        if w == 0 || h == 0 {
            let g = Glyph {
                codepoint,
                atlas_x: 0,
                atlas_y: 0,
                width: 0,
                height: 0,
                xmin: metrics.xmin as f64,
                ymin: metrics.ymin as f64,
                advance: metrics.advance_width as f64,
            };
            self.glyphs.insert(codepoint, g);
            return Some(g);
        }

        if self.pen_x + w + PADDING > self.texture_width {
            self.pen_x = PADDING;
            self.pen_y += self.row_height + PADDING;
            self.row_height = 0;
        }
        if self.pen_y + h + PADDING > self.texture_height {
            return None;
        }

        for row in 0..h {
            let dst_row = (self.pen_y + row) * self.texture_width;
            let src_row = row * w;
            for col in 0..w {
                let dst = (dst_row + self.pen_x + col) as usize;
                let src = (src_row + col) as usize;
                self.pixels[dst] = bitmap[src];
            }
        }

        let g = Glyph {
            codepoint,
            atlas_x: self.pen_x,
            atlas_y: self.pen_y,
            width: w,
            height: h,
            xmin: metrics.xmin as f64,
            ymin: metrics.ymin as f64,
            advance: metrics.advance_width as f64,
        };
        self.glyphs.insert(codepoint, g);
        self.pen_x += w + PADDING;
        if h > self.row_height {
            self.row_height = h;
        }
        self.dirty = true;
        Some(g)
    }

    pub fn build_default() -> Self {
        let mut atlas = Self::empty();
        for cp in 0x20u32..=0x7Eu32 {
            atlas.ensure(cp);
        }
        atlas
    }

    pub fn raster_size(&self) -> f64 {
        RASTER_SIZE
    }

    pub fn font_regular() -> &'static fontdue::Font {
        font::regular()
    }
}

use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::text::font::Font;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum HorizontalAlign {
    Left,
    Center,
    Right,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    Top,
    Middle,
    Bottom,
}

pub struct TextLayout<'a> {
    pub font: &'a Font,
    pub text: &'a str,
    pub size: f64,
    pub max_width: f64,
    pub h_align: HorizontalAlign,
    pub v_align: VerticalAlign,
}

impl<'a> TextLayout<'a> {
    pub fn new(font: &'a Font, text: &'a str, size: f64) -> Self {
        Self {
            font,
            text,
            size,
            max_width: f64::INFINITY,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Top,
        }
    }

    pub fn place(&self, container: Rect) -> Vec2 {
        let (text_w, text_h) = self.font.measure(self.text, self.size);
        let x = match self.h_align {
            HorizontalAlign::Left => container.x,
            HorizontalAlign::Center => container.x + (container.width - text_w) * 0.5,
            HorizontalAlign::Right => container.x + container.width - text_w,
        };
        let y = match self.v_align {
            VerticalAlign::Top => container.y,
            VerticalAlign::Middle => container.y + (container.height - text_h) * 0.5,
            VerticalAlign::Bottom => container.y + container.height - text_h,
        };
        Vec2::new(x, y)
    }
}

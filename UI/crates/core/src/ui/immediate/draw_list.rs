use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::style::palette::Rgba;

#[derive(Clone, Debug)]
pub enum DrawCommand {
    Rect {
        rect: Rect,
        color: Rgba,
        corner_radius: f64,
    },
    RectOutline {
        rect: Rect,
        color: Rgba,
        thickness: f64,
        corner_radius: f64,
    },
    Line {
        from: Vec2,
        to: Vec2,
        color: Rgba,
        thickness: f64,
    },
    Text {
        position: Vec2,
        text: String,
        color: Rgba,
        font_size: f64,
    },
    Image {
        rect: Rect,
        pixels: Vec<u8>,
        width: u32,
        height: u32,
        tint: Rgba,
    },
    Shadow {
        rect: Rect,
        color: Rgba,
        corner_radius: f64,
        spread: f64,
    },
    Clip {
        rect: Rect,
    },
    PopClip,
}

#[derive(Default)]
pub struct DrawList {
    pub commands: Vec<DrawCommand>,
}

impl DrawList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.commands.clear();
    }

    pub fn rect(&mut self, rect: Rect, color: Rgba, corner_radius: f64) {
        self.commands.push(DrawCommand::Rect {
            rect,
            color,
            corner_radius,
        });
    }

    pub fn rect_outline(&mut self, rect: Rect, color: Rgba, thickness: f64, corner_radius: f64) {
        self.commands.push(DrawCommand::RectOutline {
            rect,
            color,
            thickness,
            corner_radius,
        });
    }

    pub fn line(&mut self, from: Vec2, to: Vec2, color: Rgba, thickness: f64) {
        self.commands.push(DrawCommand::Line {
            from,
            to,
            color,
            thickness,
        });
    }

    pub fn text(&mut self, position: Vec2, text: impl Into<String>, color: Rgba, font_size: f64) {
        self.commands.push(DrawCommand::Text {
            position,
            text: text.into(),
            color,
            font_size,
        });
    }

    pub fn image(&mut self, rect: Rect, pixels: Vec<u8>, width: u32, height: u32, tint: Rgba) {
        self.commands.push(DrawCommand::Image {
            rect,
            pixels,
            width,
            height,
            tint,
        });
    }

    pub fn shadow(&mut self, rect: Rect, color: Rgba, corner_radius: f64, spread: f64) {
        self.commands.push(DrawCommand::Shadow {
            rect,
            color,
            corner_radius,
            spread,
        });
    }

    pub fn push_clip(&mut self, rect: Rect) {
        self.commands.push(DrawCommand::Clip { rect });
    }

    pub fn pop_clip(&mut self) {
        self.commands.push(DrawCommand::PopClip);
    }
}

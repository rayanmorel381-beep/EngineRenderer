use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::text::text_layout::{HorizontalAlign, VerticalAlign};

pub struct Label<'a> {
    pub text: &'a str,
    pub h_align: HorizontalAlign,
    pub v_align: VerticalAlign,
    pub muted: bool,
}

impl<'a> Label<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            text,
            h_align: HorizontalAlign::Left,
            v_align: VerticalAlign::Middle,
            muted: false,
        }
    }

    pub fn muted(mut self) -> Self {
        self.muted = true;
        self
    }

    pub fn align(mut self, h: HorizontalAlign, v: VerticalAlign) -> Self {
        self.h_align = h;
        self.v_align = v;
        self
    }

    pub fn show(self, ui: &mut UiContext, rect: Rect) {
        let color = if self.muted {
            ui.theme.palette.text_muted
        } else {
            ui.theme.palette.text
        };
        let pos = match self.h_align {
            HorizontalAlign::Left => rect.x + ui.theme.metrics.padding_small,
            HorizontalAlign::Center => rect.center().x,
            HorizontalAlign::Right => rect.x + rect.width - ui.theme.metrics.padding_small,
        };
        let y = match self.v_align {
            VerticalAlign::Top => rect.y + ui.theme.metrics.padding_small,
            VerticalAlign::Middle => rect.y + (rect.height - ui.theme.metrics.font_size_normal) * 0.5,
            VerticalAlign::Bottom => rect.y + rect.height - ui.theme.metrics.font_size_normal,
        };
        ui.draw_list
            .text(Vec2::new(pos, y), self.text, color, ui.theme.metrics.font_size_normal);
    }
}

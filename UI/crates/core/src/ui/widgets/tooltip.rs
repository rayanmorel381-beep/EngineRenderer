use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct Tooltip<'a> {
    pub text: &'a str,
}

impl<'a> Tooltip<'a> {
    pub fn new(text: &'a str) -> Self {
        Self { text }
    }

    pub fn show_at(self, ui: &mut UiContext, anchor: Vec2) {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let chars = self.text.chars().count() as f64;
        let width = chars * 7.0 + metrics.padding * 2.0;
        let height = metrics.font_size_normal + metrics.padding * 2.0;
        let rect = Rect::new(anchor.x + 12.0, anchor.y + 12.0, width, height);

        ui.draw_list.rect(rect, palette.panel_active, metrics.corner_radius);
        ui.draw_list.rect_outline(rect, palette.border, metrics.border_width, metrics.corner_radius);
        ui.draw_list.text(
            Vec2::new(rect.x + metrics.padding, rect.y + metrics.padding),
            self.text,
            palette.text,
            metrics.font_size_normal,
        );
    }
}

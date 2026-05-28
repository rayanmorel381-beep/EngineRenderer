use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct StatusBar<'a> {
    pub left: &'a str,
    pub center: &'a str,
    pub right: &'a str,
}

impl<'a> StatusBar<'a> {
    pub fn new(left: &'a str, center: &'a str, right: &'a str) -> Self {
        Self { left, center, right }
    }

    pub fn show(self, ui: &mut UiContext, rect: Rect) {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;

        ui.draw_list.rect(rect, palette.panel, 0.0);
        ui.draw_list.line(
            Vec2::new(rect.x, rect.y),
            Vec2::new(rect.x + rect.width, rect.y),
            palette.border,
            metrics.border_width,
        );

        let y = rect.y + (rect.height - metrics.font_size_small) * 0.5;
        let left_x = rect.x + metrics.padding;
        ui.draw_list.text(Vec2::new(left_x, y), self.left, palette.text_muted, metrics.font_size_small);

        let left_w = self.left.chars().count() as f64 * 7.0;
        let chars_center = self.center.chars().count() as f64;
        let chars_right = self.right.chars().count() as f64;
        let right_w = chars_right * 7.0;
        let right_x = rect.x + rect.width - right_w - metrics.padding;
        let center_w = chars_center * 7.0;
        let center_x = rect.x + (rect.width - center_w) * 0.5;
        if center_x > left_x + left_w + metrics.padding && center_x + center_w < right_x - metrics.padding {
            ui.draw_list.text(
                Vec2::new(center_x, y),
                self.center,
                palette.text_muted,
                metrics.font_size_small,
            );
        }
        if right_x > left_x + left_w + metrics.padding {
            ui.draw_list.text(
                Vec2::new(right_x, y),
                self.right,
                palette.text_muted,
                metrics.font_size_small,
            );
        }
    }
}

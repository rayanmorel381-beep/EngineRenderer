use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct ColorPicker<'a> {
    pub label: &'a str,
}

impl<'a> ColorPicker<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label }
    }

    pub fn show(self, ui: &mut UiContext, _id: WidgetId, rect: Rect, color: &mut [f64; 4]) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;

        let swatch_w = rect.height;
        let swatch = Rect::new(rect.x, rect.y, swatch_w, rect.height);
        let label_rect = Rect::new(
            rect.x + swatch_w + metrics.spacing,
            rect.y,
            (rect.width - swatch_w - metrics.spacing).max(0.0),
            rect.height,
        );

        ui.draw_list.rect(swatch, *color, metrics.corner_radius);
        ui.draw_list.rect_outline(swatch, palette.border, metrics.border_width, metrics.corner_radius);

        let text = format!(
            "{}  R{:.2} G{:.2} B{:.2} A{:.2}",
            self.label, color[0], color[1], color[2], color[3]
        );
        ui.draw_list.text(
            Vec2::new(label_rect.x, label_rect.y + (label_rect.height - metrics.font_size_normal) * 0.5),
            text,
            palette.text,
            metrics.font_size_normal,
        );

        false
    }
}

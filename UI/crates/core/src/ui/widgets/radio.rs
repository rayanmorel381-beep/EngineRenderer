use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct Radio<'a> {
    pub label: &'a str,
    pub option_value: u32,
}

impl<'a> Radio<'a> {
    pub fn new(label: &'a str, option_value: u32) -> Self {
        Self { label, option_value }
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect, current: &mut u32) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let size = metrics.row_height.min(rect.height);
        let dot_rect = Rect::new(rect.x, rect.y, size, size);

        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let hovered = rect.contains(pointer);
        let mut changed = false;

        if hovered && ui.input.pointer.left_down && ui.active != id {
            ui.set_active(id);
            if *current != self.option_value {
                *current = self.option_value;
                changed = true;
            }
        }
        if !ui.input.pointer.left_down && ui.active == id {
            ui.clear_active();
        }

        ui.draw_list.rect(dot_rect, palette.panel, size * 0.5);
        ui.draw_list.rect_outline(dot_rect, palette.border, metrics.border_width, size * 0.5);
        if *current == self.option_value {
            ui.draw_list.rect(dot_rect.shrink(3.0), palette.accent, size * 0.5);
        }

        ui.draw_list.text(
            Vec2::new(
                rect.x + size + metrics.spacing,
                rect.y + (rect.height - metrics.font_size_normal) * 0.5,
            ),
            self.label,
            palette.text,
            metrics.font_size_normal,
        );
        changed
    }
}

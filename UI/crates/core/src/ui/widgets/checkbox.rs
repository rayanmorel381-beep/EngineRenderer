use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct Checkbox<'a> {
    pub label: &'a str,
}

impl<'a> Checkbox<'a> {
    pub fn new(label: &'a str) -> Self {
        Self { label }
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect, value: &mut bool) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let box_size = metrics.row_height.min(rect.height);
        let box_rect = Rect::new(rect.x, rect.y, box_size, box_size);

        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let hovered = rect.contains(pointer);
        let mut changed = false;

        if hovered && ui.input.pointer.left_down && ui.active != id {
            ui.set_active(id);
            *value = !*value;
            changed = true;
        }
        if !ui.input.pointer.left_down && ui.active == id {
            ui.clear_active();
        }
        if hovered {
            ui.set_hovered(id);
        }

        let bg = if hovered { palette.panel_hover } else { palette.panel };
        ui.draw_list.rect(box_rect, bg, metrics.corner_radius);
        ui.draw_list.rect_outline(box_rect, palette.border, metrics.border_width, metrics.corner_radius);
        if *value {
            let inner = box_rect.shrink(3.0);
            ui.draw_list.rect(inner, palette.accent, metrics.corner_radius);
        }

        ui.draw_list.text(
            Vec2::new(
                rect.x + box_size + metrics.spacing,
                rect.y + (rect.height - metrics.font_size_normal) * 0.5,
            ),
            self.label,
            palette.text,
            metrics.font_size_normal,
        );
        changed
    }
}

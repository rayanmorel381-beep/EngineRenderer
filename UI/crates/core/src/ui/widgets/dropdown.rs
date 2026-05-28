use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct Dropdown<'a> {
    pub label: &'a str,
    pub options: &'a [&'a str],
}

impl<'a> Dropdown<'a> {
    pub fn new(label: &'a str, options: &'a [&'a str]) -> Self {
        Self { label, options }
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect, selected: &mut usize) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let hovered = rect.contains(pointer);

        let mut changed = false;
        if hovered && ui.input.pointer.left_down && ui.active != id {
            ui.set_active(id);
            *selected = (*selected + 1) % self.options.len().max(1);
            changed = true;
        }
        if !ui.input.pointer.left_down && ui.active == id {
            ui.clear_active();
        }
        if hovered {
            ui.set_hovered(id);
        }

        let bg = if hovered { palette.panel_hover } else { palette.panel };
        ui.draw_list.rect(rect, bg, metrics.corner_radius);
        ui.draw_list.rect_outline(rect, palette.border, metrics.border_width, metrics.corner_radius);

        let current = self.options.get(*selected).copied().unwrap_or("");
        let text = format!("{}: {}", self.label, current);
        ui.draw_list.text(
            Vec2::new(rect.x + metrics.padding, rect.y + (rect.height - metrics.font_size_normal) * 0.5),
            text,
            palette.text,
            metrics.font_size_normal,
        );
        changed
    }
}

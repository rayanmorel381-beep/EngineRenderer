use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct TextInput<'a> {
    pub placeholder: &'a str,
    pub multiline: bool,
}

impl<'a> TextInput<'a> {
    pub fn new() -> Self {
        Self {
            placeholder: "",
            multiline: false,
        }
    }

    pub fn placeholder(mut self, text: &'a str) -> Self {
        self.placeholder = text;
        self
    }

    pub fn multiline(mut self, enabled: bool) -> Self {
        self.multiline = enabled;
        self
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect, buffer: &mut String) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let hovered = rect.contains(pointer);

        if hovered && ui.input.pointer.left_down {
            ui.focus(id);
        }

        let mut changed = false;
        if ui.focused == id {
            let typed_text = ui.input.keyboard.drain_typed();
            if !typed_text.is_empty() {
                buffer.push_str(&typed_text);
                changed = true;
            }
        }

        let bg = if ui.focused == id {
            palette.panel_active
        } else if hovered {
            palette.panel_hover
        } else {
            palette.panel
        };
        ui.draw_list.rect(rect, bg, metrics.corner_radius);
        let border = if ui.focused == id { palette.accent } else { palette.border };
        ui.draw_list.rect_outline(rect, border, metrics.border_width, metrics.corner_radius);

        let display = if buffer.is_empty() {
            (self.placeholder.to_string(), palette.text_muted)
        } else {
            (buffer.clone(), palette.text)
        };
        ui.draw_list.text(
            Vec2::new(rect.x + metrics.padding, rect.y + (rect.height - metrics.font_size_normal) * 0.5),
            display.0,
            display.1,
            metrics.font_size_normal,
        );

        changed
    }
}

impl<'a> Default for TextInput<'a> {
    fn default() -> Self {
        Self::new()
    }
}

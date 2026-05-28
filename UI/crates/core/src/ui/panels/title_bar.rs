use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::style::icons::Icon;

pub struct TitleBar<'a> {
    pub title: &'a str,
    pub icon: Icon,
    pub closable: bool,
}

impl<'a> TitleBar<'a> {
    pub fn new(title: &'a str) -> Self {
        Self {
            title,
            icon: Icon::None,
            closable: true,
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn show(self, ui: &mut UiContext, rect: Rect) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;

        ui.draw_list.rect(rect, palette.panel_active, metrics.corner_radius);
        let label = if self.icon == Icon::None {
            self.title.to_string()
        } else {
            format!("{}  {}", self.icon.glyph(), self.title)
        };
        ui.draw_list.text(
            Vec2::new(rect.x + metrics.padding, rect.y + (rect.height - metrics.font_size_normal) * 0.5),
            label,
            palette.text,
            metrics.font_size_normal,
        );

        if !self.closable {
            return false;
        }
        let close_size = rect.height - metrics.padding_small * 2.0;
        let close_rect = Rect::new(
            rect.x + rect.width - close_size - metrics.padding_small,
            rect.y + metrics.padding_small,
            close_size,
            close_size,
        );
        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let hovered = close_rect.contains(pointer);
        let bg = if hovered { palette.error } else { palette.panel_hover };
        ui.draw_list.rect(close_rect, bg, metrics.corner_radius);
        ui.draw_list.text(
            Vec2::new(close_rect.x + 4.0, close_rect.y + 2.0),
            Icon::Cross.glyph(),
            palette.text,
            metrics.font_size_normal,
        );
        hovered && ui.input.pointer.left_down
    }
}

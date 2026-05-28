use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::style::icons::Icon;

pub struct ContextMenuItem<'a> {
    pub label: &'a str,
    pub icon: Icon,
    pub shortcut: &'a str,
    pub enabled: bool,
}

impl<'a> ContextMenuItem<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            icon: Icon::None,
            shortcut: "",
            enabled: true,
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    pub fn with_shortcut(mut self, shortcut: &'a str) -> Self {
        self.shortcut = shortcut;
        self
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

pub struct ContextMenu<'a> {
    pub items: &'a [ContextMenuItem<'a>],
}

impl<'a> ContextMenu<'a> {
    pub fn new(items: &'a [ContextMenuItem<'a>]) -> Self {
        Self { items }
    }

    pub fn show_at(self, ui: &mut UiContext, anchor: Vec2) -> Option<usize> {
        if self.items.is_empty() {
            return None;
        }
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let row_h = metrics.row_height;
        let width: f64 = 220.0;
        let height = row_h * self.items.len() as f64 + metrics.padding_small * 2.0;
        let rect = Rect::new(anchor.x, anchor.y, width, height);

        ui.draw_list.rect(rect, palette.panel_active, metrics.corner_radius);
        ui.draw_list.rect_outline(rect, palette.border, metrics.border_width, metrics.corner_radius);

        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let mut chosen: Option<usize> = None;

        for (i, item) in self.items.iter().enumerate() {
            let row = Rect::new(
                rect.x,
                rect.y + metrics.padding_small + row_h * i as f64,
                rect.width,
                row_h,
            );
            let hovered = item.enabled && row.contains(pointer);
            if hovered {
                ui.draw_list.rect(row, palette.panel_hover, 0.0);
                if ui.input.pointer.left_down {
                    chosen = Some(i);
                }
            }

            let color = if item.enabled {
                palette.text
            } else {
                palette.text_disabled
            };
            let icon_glyph = item.icon.glyph();
            let label = if icon_glyph.is_empty() {
                item.label.to_string()
            } else {
                format!("{}  {}", icon_glyph, item.label)
            };
            ui.draw_list.text(
                Vec2::new(row.x + metrics.padding, row.y + (row.height - metrics.font_size_normal) * 0.5),
                label,
                color,
                metrics.font_size_normal,
            );

            if !item.shortcut.is_empty() {
                let chars = item.shortcut.chars().count() as f64;
                ui.draw_list.text(
                    Vec2::new(
                        row.x + row.width - chars * 7.0 - metrics.padding,
                        row.y + (row.height - metrics.font_size_normal) * 0.5,
                    ),
                    item.shortcut,
                    palette.text_muted,
                    metrics.font_size_normal,
                );
            }
        }

        chosen
    }
}

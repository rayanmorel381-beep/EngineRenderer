use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;

#[derive(Clone, Debug)]
pub struct ContentBrowserItem {
    pub name: String,
    pub icon: Icon,
}

impl ContentBrowserItem {
    pub fn new(name: impl Into<String>, icon: Icon) -> Self {
        Self {
            name: name.into(),
            icon,
        }
    }
}

pub struct ContentBrowserView {
    pub items: Vec<ContentBrowserItem>,
    pub cell_size: f64,
    pub selected: Option<usize>,
    pub scroll_y: f64,
}

impl Default for ContentBrowserView {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            cell_size: 84.0,
            selected: None,
            scroll_y: 0.0,
        }
    }
}

impl ContentBrowserView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Content Browser").with_icon(Icon::Folder);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);

        let dense = body.width < 420.0;
        let cell = self.cell_size.min((body.width - metrics.spacing * 2.0).max(56.0)).max(if dense { 60.0 } else { 72.0 });
        let cols = ((body.width / (cell + metrics.spacing)).floor() as usize).max(1);
        let rows = self.items.len().div_ceil(cols);
        let content_h = rows as f64 * (cell + metrics.spacing) - metrics.spacing;
        let max_scroll = (content_h - body.height).max(0.0);
        self.scroll_y = self.scroll_y.clamp(0.0, max_scroll);
        ui.draw_list.push_clip(body);
        if self.items.is_empty() {
            ui.draw_list.text(
                Vec2::new(body.x + metrics.padding, body.y + metrics.padding),
                "No imported assets",
                palette.text_muted,
                metrics.font_size_normal,
            );
            ui.draw_list.pop_clip();
            return;
        }
        for (i, item) in self.items.iter().enumerate() {
            let row = i / cols;
            let col = i % cols;
            let x = body.x + col as f64 * (cell + metrics.spacing);
            let y = body.y + row as f64 * (cell + metrics.spacing) - self.scroll_y;
            if y + cell < body.y {
                continue;
            }
            if y > body.y + body.height {
                break;
            }
            let cell_rect = Rect::new(x, y, cell, cell);
            let hovered = cell_rect.contains(pointer);
            let id = WidgetId::hash_str(&item.name);

            let bg = if Some(i) == self.selected {
                palette.selection
            } else if hovered {
                palette.panel_hover
            } else {
                palette.panel
            };
            ui.draw_list.rect(cell_rect, bg, metrics.corner_radius);
            ui.draw_list.rect_outline(cell_rect, palette.border, metrics.border_width, metrics.corner_radius);

            let icon_rect = Rect::new(cell_rect.x, cell_rect.y, cell_rect.width, cell_rect.height - 22.0);
            let glyph = item.icon.glyph();
            ui.draw_list.text(
                Vec2::new(icon_rect.x + icon_rect.width * 0.5 - metrics.font_size_large * 0.35, icon_rect.y + icon_rect.height * 0.5 - metrics.font_size_large * 0.45),
                glyph,
                palette.text,
                metrics.font_size_large,
            );
            ui.draw_list.text(
                Vec2::new(cell_rect.x + 4.0, cell_rect.y + cell_rect.height - 16.0),
                &item.name,
                palette.text_muted,
                metrics.font_size_small,
            );

            if hovered && ui.input.pointer.left_down && ui.active != id {
                ui.set_active(id);
                self.selected = Some(i);
            }
            if !ui.input.pointer.left_down && ui.active == id {
                ui.clear_active();
            }
        }
        ui.draw_list.pop_clip();
    }
}

use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::style::icons::Icon;

pub struct ToolBarItem<'a> {
    pub icon: Icon,
    pub label: &'a str,
    pub tooltip: &'a str,
    pub active: bool,
}

impl<'a> ToolBarItem<'a> {
    pub fn new(icon: Icon, tooltip: &'a str) -> Self {
        Self {
            icon,
            label: "",
            tooltip,
            active: false,
        }
    }

    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = label;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

pub struct ToolBar<'a> {
    pub items: &'a [ToolBarItem<'a>],
}

impl<'a> ToolBar<'a> {
    pub fn new(items: &'a [ToolBarItem<'a>]) -> Self {
        Self { items }
    }

    pub fn show(self, ui: &mut UiContext, rect: Rect) -> Option<usize> {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;

        ui.draw_list.rect(rect, palette.panel, 0.0);
        ui.draw_list.line(
            Vec2::new(rect.x, rect.y + rect.height),
            Vec2::new(rect.x + rect.width, rect.y + rect.height),
            palette.border,
            metrics.border_width,
        );

        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let mut chosen: Option<usize> = None;
        let mut x = rect.x + metrics.padding;
        let item_h = rect.height - metrics.padding_small * 2.0;
        let available_w = (rect.width - metrics.padding * 2.0).max(0.0);
        let spacing_total = metrics.spacing * self.items.len().saturating_sub(1) as f64;
        let dense = available_w < self.items.len() as f64 * (item_h + metrics.spacing + 28.0);
        let base_item_w = if dense {
            item_h
        } else {
            item_h + if self.items.iter().any(|i| !i.label.is_empty()) { 60.0 } else { 0.0 }
        };
        let item_w = ((available_w - spacing_total) / self.items.len().max(1) as f64)
            .min(base_item_w)
            .max(18.0);

        for (i, item) in self.items.iter().enumerate() {
            if x >= rect.x + rect.width {
                break;
            }
            let item_rect = Rect::new(x, rect.y + metrics.padding_small, item_w, item_h);
            let hovered = item_rect.contains(pointer);
            let id = WidgetId::hash_str(item.tooltip).child(item.label);

            let bg = if item.active {
                palette.accent
            } else if hovered {
                palette.panel_hover
            } else {
                palette.panel
            };
            ui.draw_list.rect(item_rect, bg, metrics.corner_radius);
            let label_text = if dense || item.label.is_empty() {
                item.icon.glyph().to_string()
            } else {
                format!("{} {}", item.icon.glyph(), item.label)
            };
            ui.draw_list.text(
                Vec2::new(item_rect.x + metrics.padding_small, item_rect.y + (item_rect.height - metrics.font_size_normal) * 0.5),
                label_text,
                palette.text,
                metrics.font_size_normal,
            );

            if hovered && ui.input.pointer.left_down && ui.active != id {
                ui.set_active(id);
                chosen = Some(i);
            }
            if !ui.input.pointer.left_down && ui.active == id {
                ui.clear_active();
            }
            x += item_w + metrics.spacing;
        }
        chosen
    }
}

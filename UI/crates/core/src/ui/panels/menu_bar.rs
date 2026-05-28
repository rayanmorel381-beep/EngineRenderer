use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct MenuItem<'a> {
    pub label: &'a str,
    pub children: &'a [&'a str],
}

impl<'a> MenuItem<'a> {
    pub fn new(label: &'a str, children: &'a [&'a str]) -> Self {
        Self { label, children }
    }
}

pub struct MenuBar<'a> {
    pub items: &'a [MenuItem<'a>],
}

impl<'a> MenuBar<'a> {
    pub fn new(items: &'a [MenuItem<'a>]) -> Self {
        Self { items }
    }

    pub fn show(self, ui: &mut UiContext, rect: Rect) -> Option<(usize, usize)> {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);

        ui.draw_list.rect(rect, palette.panel, 0.0);
        ui.draw_list.line(
            Vec2::new(rect.x, rect.y + rect.height),
            Vec2::new(rect.x + rect.width, rect.y + rect.height),
            palette.border,
            metrics.border_width,
        );

        let mut chosen: Option<(usize, usize)> = None;
        let mut x = rect.x + metrics.padding;
        let max_item_w = ((rect.width - metrics.padding * 2.0) / self.items.len().max(1) as f64).max(0.0);
        for (i, item) in self.items.iter().enumerate() {
            let chars = item.label.chars().count() as f64;
            let w = (chars * 7.5 + metrics.padding * 2.0).min(max_item_w);
            if w <= 0.0 || x >= rect.x + rect.width {
                break;
            }
            let item_rect = Rect::new(x, rect.y, w, rect.height);
            let hovered = item_rect.contains(pointer);
            let id = WidgetId::hash_str(item.label);

            if hovered {
                ui.draw_list.rect(item_rect, palette.panel_hover, 0.0);
                ui.set_hovered(id);
                if ui.input.pointer.left_down && !item.children.is_empty() {
                    for (j, child) in item.children.iter().enumerate() {
                        let _ = j;
                        let _ = child;
                    }
                    chosen = Some((i, 0));
                }
            }
            ui.draw_list.text(
                Vec2::new(item_rect.x + metrics.padding, item_rect.y + (item_rect.height - metrics.font_size_normal) * 0.5),
                item.label,
                palette.text,
                metrics.font_size_normal,
            );
            x += w;
        }
        chosen
    }
}

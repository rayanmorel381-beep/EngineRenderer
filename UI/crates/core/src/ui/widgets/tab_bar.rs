use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct Tab<'a> {
    pub label: &'a str,
    pub closable: bool,
}

impl<'a> Tab<'a> {
    pub fn new(label: &'a str) -> Self {
        Self {
            label,
            closable: false,
        }
    }

    pub fn closable(mut self, value: bool) -> Self {
        self.closable = value;
        self
    }
}

pub struct TabBar<'a> {
    pub tabs: &'a [Tab<'a>],
}

impl<'a> TabBar<'a> {
    pub fn new(tabs: &'a [Tab<'a>]) -> Self {
        Self { tabs }
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect, active: &mut usize) -> bool {
        if self.tabs.is_empty() {
            return false;
        }
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let mut changed = false;
        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);

        let tab_w = (rect.width / self.tabs.len() as f64).max(0.0);
        for (i, tab) in self.tabs.iter().enumerate() {
            let tab_rect = Rect::new(rect.x + i as f64 * tab_w, rect.y, tab_w, rect.height);
            if tab_rect.width <= 0.0 {
                continue;
            }
            let hovered = tab_rect.contains(pointer);
            let is_active = *active == i;
            let tab_id = id.child(tab.label);

            if hovered && ui.input.pointer.left_down && ui.active != tab_id {
                ui.set_active(tab_id);
                if !is_active {
                    *active = i;
                    changed = true;
                }
            }
            if !ui.input.pointer.left_down && ui.active == tab_id {
                ui.clear_active();
            }

            let bg = if is_active {
                palette.panel_active
            } else if hovered {
                palette.panel_hover
            } else {
                palette.panel
            };
            ui.draw_list.rect(tab_rect, bg, metrics.corner_radius);
            if is_active {
                let underline = Rect::new(
                    tab_rect.x,
                    tab_rect.y + tab_rect.height - 2.0,
                    tab_rect.width,
                    2.0,
                );
                ui.draw_list.rect(underline, palette.accent, 0.0);
            }
            ui.draw_list.text(
                Vec2::new(
                    tab_rect.x + metrics.padding,
                    tab_rect.y + (tab_rect.height - metrics.font_size_normal) * 0.5,
                ),
                tab.label,
                if is_active { palette.text } else { palette.text_muted },
                metrics.font_size_normal,
            );
        }
        changed
    }
}

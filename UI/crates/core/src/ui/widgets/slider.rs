use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct Slider<'a> {
    pub label: &'a str,
    pub min: f64,
    pub max: f64,
    pub step: f64,
    pub format: &'a str,
}

impl<'a> Slider<'a> {
    pub fn new(label: &'a str, min: f64, max: f64) -> Self {
        Self {
            label,
            min,
            max,
            step: 0.0,
            format: "{:.3}",
        }
    }

    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    pub fn format(mut self, format: &'a str) -> Self {
        self.format = format;
        self
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect, value: &mut f64) -> bool {
        let palette = ui.theme.palette;
        let metrics = ui.theme.metrics;

        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let hovered = rect.contains(pointer);
        let pressed = hovered && ui.input.pointer.left_down;

        let mut changed = false;
        if pressed {
            ui.set_active(id);
        }
        if ui.active == id && ui.input.pointer.left_down {
            let ratio = ((pointer.x - rect.x) / rect.width.max(1.0)).clamp(0.0, 1.0);
            let mut new_value = self.min + (self.max - self.min) * ratio;
            if self.step > 0.0 {
                new_value = (new_value / self.step).round() * self.step;
            }
            new_value = new_value.clamp(self.min, self.max);
            if (new_value - *value).abs() > f64::EPSILON {
                *value = new_value;
                changed = true;
            }
        }
        if !ui.input.pointer.left_down && ui.active == id {
            ui.clear_active();
        }
        if hovered {
            ui.set_hovered(id);
        }

        let ratio = ((*value - self.min) / (self.max - self.min).max(f64::EPSILON))
            .clamp(0.0, 1.0);

        ui.draw_list.rect(rect, palette.panel, metrics.corner_radius);
        let fill_rect = Rect::new(rect.x, rect.y, rect.width * ratio, rect.height);
        ui.draw_list.rect(fill_rect, palette.accent, metrics.corner_radius);
        ui.draw_list.rect_outline(rect, palette.border, metrics.border_width, metrics.corner_radius);

        let text = format!("{}: {:.3}", self.label, *value);
        ui.draw_list.text(
            Vec2::new(rect.x + metrics.padding, rect.y + (rect.height - metrics.font_size_normal) * 0.5),
            text,
            palette.text,
            metrics.font_size_normal,
        );

        changed
    }
}

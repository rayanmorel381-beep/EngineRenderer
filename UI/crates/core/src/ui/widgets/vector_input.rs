use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::widgets::slider::Slider;

pub struct VectorInput<'a> {
    pub label: &'a str,
    pub min: f64,
    pub max: f64,
    pub components: usize,
}

impl<'a> VectorInput<'a> {
    pub fn new(label: &'a str, components: usize) -> Self {
        Self {
            label,
            min: -1000.0,
            max: 1000.0,
            components: components.clamp(1, 4),
        }
    }

    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    pub fn show(self, ui: &mut UiContext, id: WidgetId, rect: Rect, values: &mut [f64]) -> bool {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let label_w = 80.0;
        ui.draw_list.text(
            Vec2::new(rect.x, rect.y + (rect.height - metrics.font_size_normal) * 0.5),
            self.label,
            palette.text,
            metrics.font_size_normal,
        );

        let count = self.components.min(values.len()).max(1);
        let avail = (rect.width - label_w - metrics.spacing * count as f64).max(0.0);
        let cell_w = avail / count as f64;

        let mut changed = false;
        for (i, value) in values.iter_mut().take(count).enumerate() {
            let x = rect.x + label_w + (cell_w + metrics.spacing) * i as f64;
            let cell = Rect::new(x, rect.y, cell_w, rect.height);
            let axis_label = ["X", "Y", "Z", "W"][i.min(3)];
            let component_id = id.child(axis_label);
            let slider = Slider::new(axis_label, self.min, self.max);
            if slider.show(ui, component_id, cell, value) {
                changed = true;
            }
        }
        changed
    }
}

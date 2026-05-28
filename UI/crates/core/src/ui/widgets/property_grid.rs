use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};

#[derive(Clone, Debug)]
pub struct PropertyRow {
    pub label: String,
    pub value: String,
    pub editable: bool,
}

impl PropertyRow {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            editable: false,
        }
    }

    pub fn editable(mut self, editable: bool) -> Self {
        self.editable = editable;
        self
    }
}

pub struct PropertyGrid<'a> {
    pub rows: &'a [PropertyRow],
    pub label_ratio: f64,
}

impl<'a> PropertyGrid<'a> {
    pub fn new(rows: &'a [PropertyRow]) -> Self {
        Self {
            rows,
            label_ratio: 0.4,
        }
    }

    pub fn show(self, ui: &mut UiContext, _id: WidgetId, rect: Rect) {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let row_h = metrics.row_height;
        let label_w = rect.width * self.label_ratio;
        let value_w = rect.width - label_w;

        for (i, row) in self.rows.iter().enumerate() {
            let y = rect.y + i as f64 * row_h;
            if y + row_h > rect.y + rect.height {
                break;
            }
            if i % 2 == 1 {
                let stripe = Rect::new(rect.x, y, rect.width, row_h);
                ui.draw_list.rect(stripe, palette.panel, 0.0);
            }
            ui.draw_list.text(
                Vec2::new(rect.x + metrics.padding, y + (row_h - metrics.font_size_normal) * 0.5),
                &row.label,
                palette.text_muted,
                metrics.font_size_normal,
            );
            ui.draw_list.text(
                Vec2::new(
                    rect.x + label_w + metrics.padding,
                    y + (row_h - metrics.font_size_normal) * 0.5,
                ),
                &row.value,
                if row.editable { palette.text } else { palette.text_muted },
                metrics.font_size_normal,
            );
            let _ = value_w;
        }
    }
}

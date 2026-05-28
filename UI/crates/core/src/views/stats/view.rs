use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;

pub struct StatsView {
    pub frame_micros_history: Vec<u64>,
    pub history_capacity: usize,
}

impl Default for StatsView {
    fn default() -> Self {
        Self {
            frame_micros_history: Vec::new(),
            history_capacity: 240,
        }
    }
}

impl StatsView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_frame(&mut self, micros: u64) {
        if self.frame_micros_history.len() >= self.history_capacity {
            self.frame_micros_history.remove(0);
        }
        self.frame_micros_history.push(micros);
    }

    pub fn fps(&self) -> f64 {
        if self.frame_micros_history.is_empty() {
            return 0.0;
        }
        let sum: u64 = self.frame_micros_history.iter().copied().sum();
        let avg = sum as f64 / self.frame_micros_history.len() as f64;
        if avg <= 0.0 {
            0.0
        } else {
            1_000_000.0 / avg
        }
    }

    pub fn show(&self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Stats").with_icon(Icon::Info);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;

        ui.draw_list.text(
            Vec2::new(body.x + metrics.padding, body.y + metrics.padding),
            format!("FPS: {:.1}", self.fps()),
            palette.text,
            metrics.font_size_large,
        );
        ui.draw_list.text(
            Vec2::new(body.x + metrics.padding, body.y + metrics.padding + 22.0),
            format!("Samples: {}", self.frame_micros_history.len()),
            palette.text_muted,
            metrics.font_size_normal,
        );

        let graph = Rect::new(
            body.x + metrics.padding,
            body.y + metrics.padding + 48.0,
            (body.width - metrics.padding * 2.0).max(0.0),
            (body.height - metrics.padding * 2.0 - 48.0).max(0.0),
        );
        if graph.width <= 0.0 || graph.height <= 0.0 || self.frame_micros_history.is_empty() {
            return;
        }
        ui.draw_list.rect(graph, palette.panel, metrics.corner_radius);
        let max_micros = self.frame_micros_history.iter().copied().max().unwrap_or(1) as f64;
        let count = self.frame_micros_history.len() as f64;
        let bar_w = graph.width / count;
        for (i, micros) in self.frame_micros_history.iter().enumerate() {
            let ratio = (*micros as f64 / max_micros).clamp(0.0, 1.0);
            let h = graph.height * ratio;
            let bar = Rect::new(
                graph.x + i as f64 * bar_w,
                graph.y + graph.height - h,
                bar_w.max(1.0),
                h,
            );
            ui.draw_list.rect(bar, palette.accent, 0.0);
        }
    }
}

use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle};

#[derive(Clone, Debug, Default)]
pub struct FrameSample {
    pub cpu_ms: f64,
    pub gpu_ms: f64,
    pub draw_calls: u32,
    pub triangles: u32,
    pub texture_mb: f64,
}

pub struct ProfilerView {
    pub history: Vec<FrameSample>,
    pub capacity: usize,
    pub paused: bool,
}

impl Default for ProfilerView {
    fn default() -> Self {
        Self { history: Vec::new(), capacity: 120, paused: false }
    }
}

impl ProfilerView {
    pub fn new() -> Self { Self::default() }

    pub fn push(&mut self, sample: FrameSample) {
        if self.paused { return; }
        self.history.push(sample);
        if self.history.len() > self.capacity { self.history.remove(0); }
    }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Profiler").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("profiler");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let pause_lbl = if self.paused { "Reprendre" } else { "Pause" };
        if Button::new(pause_lbl).with_style(ButtonStyle::Secondary)
            .show(ui, id.child("pause"), Rect::new(lx, y, 90.0, row_h)).clicked { self.paused = !self.paused; }
        if Button::new("Effacer").with_style(ButtonStyle::Danger)
            .show(ui, id.child("clear"), Rect::new(lx + 96.0, y, 70.0, row_h)).clicked { self.history.clear(); }
        y += row_h + sp;

        let last = self.history.last().cloned().unwrap_or_default();
        let fps = if last.cpu_ms > 0.0 { 1000.0 / last.cpu_ms } else { 0.0 };

        ui.draw_list.text(Vec2::new(lx, y), &format!("CPU: {:.2} ms  |  GPU: {:.2} ms  |  FPS: {:.0}", last.cpu_ms, last.gpu_ms, fps), p.text, m.font_size_normal);
        y += m.font_size_normal + sp;
        ui.draw_list.text(Vec2::new(lx, y), &format!("Draw Calls: {}  |  Triangles: {}  |  Textures: {:.1} MB", last.draw_calls, last.triangles, last.texture_mb), p.text_muted, m.font_size_normal);
        y += m.font_size_normal + sp * 2.0;

        let graph_h = (body.height - (y - body.y) - pad).max(50.0);
        let graph_r = Rect::new(lx, y, w, graph_h);
        ui.draw_list.rect(graph_r, p.panel, 4.0);

        if !self.history.is_empty() {
            let max_ms = self.history.iter().map(|s| s.cpu_ms.max(s.gpu_ms)).fold(0.01f64, f64::max);
            let n = self.history.len();
            let bar_w = (w / self.capacity as f64).max(1.0);
            for (i, sample) in self.history.iter().enumerate() {
                let cpu_h = (sample.cpu_ms / max_ms * graph_h).clamp(1.0, graph_h);
                let gpu_h = (sample.gpu_ms / max_ms * graph_h).clamp(1.0, graph_h);
                let bx = lx + (self.capacity - n + i) as f64 * bar_w;
                ui.draw_list.rect(Rect::new(bx, y + graph_h - cpu_h, bar_w * 0.5 - 0.5, cpu_h), p.accent, 0.0);
                ui.draw_list.rect(Rect::new(bx + bar_w * 0.5, y + graph_h - gpu_h, bar_w * 0.5 - 0.5, gpu_h), p.warning, 0.0);
            }
        }

        let legend_y = y + graph_h + 4.0;
        ui.draw_list.rect(Rect::new(lx, legend_y, 10.0, 10.0), p.accent, 0.0);
        ui.draw_list.text(Vec2::new(lx + 14.0, legend_y - 1.0), "CPU", p.text_muted, m.font_size_normal);
        ui.draw_list.rect(Rect::new(lx + 50.0, legend_y, 10.0, 10.0), p.warning, 0.0);
        ui.draw_list.text(Vec2::new(lx + 64.0, legend_y - 1.0), "GPU", p.text_muted, m.font_size_normal);
    }
}

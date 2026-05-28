use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

#[derive(Clone, Debug)]
pub struct CurveKey {
    pub time: f64,
    pub value: f64,
    pub tangent_in: f64,
    pub tangent_out: f64,
}

impl CurveKey {
    pub fn new(time: f64, value: f64) -> Self {
        Self { time, value, tangent_in: 0.0, tangent_out: 0.0 }
    }
}

#[derive(Clone, Debug)]
pub struct EditableCurve {
    pub name: String,
    pub keys: Vec<CurveKey>,
    pub color: [f64; 4],
    pub enabled: bool,
}

impl EditableCurve {
    pub fn new(name: impl Into<String>, color: [f64; 4]) -> Self {
        Self { name: name.into(), keys: Vec::new(), color, enabled: true }
    }

    pub fn evaluate(&self, t: f64) -> f64 {
        if self.keys.is_empty() { return 0.0; }
        if self.keys.len() == 1 { return self.keys[0].value; }
        if t <= self.keys[0].time { return self.keys[0].value; }
        let last = &self.keys[self.keys.len() - 1];
        if t >= last.time { return last.value; }
        let seg = self.keys.windows(2).find(|w| t >= w[0].time && t <= w[1].time);
        if let Some(seg) = seg {
            let k0 = &seg[0];
            let k1 = &seg[1];
            let dt = k1.time - k0.time;
            if dt < 1e-12 { return k0.value; }
            let u = (t - k0.time) / dt;
            let u2 = u * u;
            let u3 = u2 * u;
            let h00 = 2.0 * u3 - 3.0 * u2 + 1.0;
            let h10 = u3 - 2.0 * u2 + u;
            let h01 = -2.0 * u3 + 3.0 * u2;
            let h11 = u3 - u2;
            h00 * k0.value + h10 * dt * k0.tangent_out + h01 * k1.value + h11 * dt * k1.tangent_in
        } else {
            0.0
        }
    }
}

pub struct CurveEditorView {
    pub curves: Vec<EditableCurve>,
    pub selected_curve: Option<usize>,
    pub selected_key: Option<usize>,
    pub scroll: [f64; 2],
    pub zoom: [f64; 2],
    pub show_grid: bool,
    pub time_min: f64,
    pub time_max: f64,
    pub value_min: f64,
    pub value_max: f64,
}

impl Default for CurveEditorView {
    fn default() -> Self {
        let mut view = Self {
            curves: Vec::new(),
            selected_curve: None,
            selected_key: None,
            scroll: [0.0; 2],
            zoom: [1.0; 2],
            show_grid: true,
            time_min: 0.0,
            time_max: 1.0,
            value_min: 0.0,
            value_max: 1.0,
        };
        let mut c0 = EditableCurve::new("Alpha", [0.9, 0.6, 0.1, 1.0]);
        c0.keys.push(CurveKey::new(0.0, 0.0));
        c0.keys.push(CurveKey { time: 0.3, value: 1.0, tangent_in: 0.0, tangent_out: 0.0 });
        c0.keys.push(CurveKey::new(1.0, 0.5));
        view.curves.push(c0);
        let mut c1 = EditableCurve::new("Scale", [0.3, 0.7, 1.0, 1.0]);
        c1.keys.push(CurveKey::new(0.0, 1.0));
        c1.keys.push(CurveKey::new(0.5, 0.5));
        c1.keys.push(CurveKey::new(1.0, 0.0));
        view.curves.push(c1);
        view
    }
}

impl CurveEditorView {
    pub fn new() -> Self { Self::default() }

    fn to_canvas(&self, canvas: Rect, t: f64, v: f64) -> Vec2 {
        let tx = (t - self.time_min) / (self.time_max - self.time_min).max(1e-6);
        let ty = 1.0 - (v - self.value_min) / (self.value_max - self.value_min).max(1e-6);
        Vec2::new(canvas.x + tx * canvas.width, canvas.y + ty * canvas.height)
    }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Éditeur de courbes").with_icon(Icon::Curve);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("curve_ed");

        let (sidebar_r, canvas_r) = body.split_left(160.0);
        let slx = sidebar_r.x + pad;
        let sw = sidebar_r.width - pad * 2.0;
        let mut sy = sidebar_r.y + pad;

        ui.draw_list.text(Vec2::new(slx, sy), "Courbes", p.text, m.font_size_normal);
        sy += m.font_size_normal + sp;

        let grid_lbl = if self.show_grid { "Grille: ON" } else { "Grille: OFF" };
        if Button::new(grid_lbl).with_style(if self.show_grid { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("grid"), Rect::new(slx, sy, sw, row_h)).clicked {
            self.show_grid = !self.show_grid;
        }
        sy += row_h + sp;
        Slider::new("T.début", -10.0, 10.0).show(ui, id.child("tmin"), Rect::new(slx, sy, sw, row_h), &mut self.time_min);
        sy += row_h + sp;
        Slider::new("T.fin", 0.1, 100.0).show(ui, id.child("tmax"), Rect::new(slx, sy, sw, row_h), &mut self.time_max);
        sy += row_h + sp;

        let mut remove_curve: Option<usize> = None;
        for (ci, curve) in self.curves.iter().enumerate() {
            let sel = self.selected_curve == Some(ci);
            let cr = Rect::new(slx, sy, sw, row_h);
            ui.draw_list.rect(cr, if sel { p.panel_active } else { p.panel }, 2.0);
            let dot_r = Rect::new(slx + 2.0, sy + (row_h - 8.0) * 0.5, 8.0, 8.0);
            ui.draw_list.rect(dot_r, curve.color, 2.0);
            ui.draw_list.text(Vec2::new(slx + 14.0, sy + (row_h - m.font_size_small) * 0.5), &curve.name, p.text, m.font_size_small);
            let del_r = Rect::new(slx + sw - 20.0, sy + 2.0, 18.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("cdel{ci}")), del_r).clicked {
                remove_curve = Some(ci);
            }
            if ui.is_rect_hovered(cr) {
                self.selected_curve = Some(ci);
                self.selected_key = None;
            }
            sy += row_h + 2.0;
        }
        if let Some(idx) = remove_curve {
            self.curves.remove(idx);
            if self.selected_curve == Some(idx) { self.selected_curve = None; self.selected_key = None; }
        }
        if Button::new("+ Courbe").with_style(ButtonStyle::Secondary).show(ui, id.child("add_c"), Rect::new(slx, sy, sw, row_h)).clicked {
            let n = self.curves.len();
            let colors = [[0.9, 0.3, 0.3, 1.0], [0.3, 0.9, 0.3, 1.0], [0.3, 0.3, 0.9, 1.0], [0.9, 0.9, 0.3, 1.0]];
            let col = colors[n % colors.len()];
            let mut c = EditableCurve::new(format!("Courbe {}", n + 1), col);
            c.keys.push(CurveKey::new(0.0, 0.0));
            c.keys.push(CurveKey::new(1.0, 1.0));
            self.curves.push(c);
        }
        sy += row_h + sp;

        if let Some(cidx) = self.selected_curve {
            if cidx < self.curves.len() {
                let curve = &self.curves[cidx];
                ui.draw_list.text(Vec2::new(slx, sy), &format!("{} points", curve.keys.len()), p.text_muted, m.font_size_small);
                sy += m.font_size_small + sp;
                if let Some(kidx) = self.selected_key {
                    if kidx < curve.keys.len() {
                        let key = &curve.keys[kidx];
                        let t_lbl = format!("T: {:.3}  V: {:.3}", key.time, key.value);
                        ui.draw_list.text(Vec2::new(slx, sy), &t_lbl, p.text, m.font_size_small);
                        sy += m.font_size_small + sp;
                        let tang_lbl = format!("In: {:.2}  Out: {:.2}", key.tangent_in, key.tangent_out);
                        ui.draw_list.text(Vec2::new(slx, sy), &tang_lbl, p.text_muted, m.font_size_small);
                    }
                }
            }
        }

        let pad_cv = 20.0;
        let canvas = Rect::new(canvas_r.x + pad_cv, canvas_r.y + pad_cv, canvas_r.width - pad_cv * 2.0, canvas_r.height - pad_cv * 2.0);
        ui.draw_list.rect(canvas_r, p.viewport_clear, 0.0);
        ui.draw_list.rect_outline(canvas, p.border, 2.0, 1.0);

        if self.show_grid {
            let grid_lines = 8;
            for gi in 0..=grid_lines {
                let t = gi as f64 / grid_lines as f64;
                let gx = canvas.x + t * canvas.width;
                let gy = canvas.y + t * canvas.height;
                ui.draw_list.line(Vec2::new(gx, canvas.y), Vec2::new(gx, canvas.y + canvas.height), [0.3, 0.3, 0.3, 0.5], 0.5);
                ui.draw_list.line(Vec2::new(canvas.x, gy), Vec2::new(canvas.x + canvas.width, gy), [0.3, 0.3, 0.3, 0.5], 0.5);
            }
        }

        for curve in &self.curves {
            if !curve.enabled { continue; }
            let steps = 80;
            for si in 0..steps {
                let t0 = self.time_min + si as f64 / steps as f64 * (self.time_max - self.time_min);
                let t1 = self.time_min + (si + 1) as f64 / steps as f64 * (self.time_max - self.time_min);
                let v0 = curve.evaluate(t0);
                let v1 = curve.evaluate(t1);
                let p0 = self.to_canvas(canvas, t0, v0);
                let p1 = self.to_canvas(canvas, t1, v1);
                ui.draw_list.line(p0, p1, curve.color, 1.5);
            }

            for (ki, key) in curve.keys.iter().enumerate() {
                let kp = self.to_canvas(canvas, key.time, key.value);
                let kr = Rect::new(kp.x - 5.0, kp.y - 5.0, 10.0, 10.0);
                let sel_k = self.selected_curve.is_some() && self.selected_key == Some(ki);
                let kcol = if sel_k { [1.0, 1.0, 0.5, 1.0] } else { curve.color };
                ui.draw_list.rect(kr, kcol, 2.0);
                if ui.is_rect_hovered(kr) {
                    self.selected_key = Some(ki);
                }
            }
        }

        let add_key_r = Rect::new(canvas_r.x + canvas_r.width - 90.0, canvas_r.y + canvas_r.height - row_h - pad, 85.0, row_h);
        if Button::new("+ Point").with_style(ButtonStyle::Secondary).show(ui, id.child("add_k"), add_key_r).clicked {
            if let Some(cidx) = self.selected_curve {
                if cidx < self.curves.len() {
                    let t = self.curves[cidx].keys.last().map(|k| k.time + 0.2).unwrap_or(0.0).min(self.time_max);
                    let v = 0.5;
                    self.curves[cidx].keys.push(CurveKey::new(t, v));
                    self.curves[cidx].keys.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(std::cmp::Ordering::Equal));
                }
            }
        }
    }
}

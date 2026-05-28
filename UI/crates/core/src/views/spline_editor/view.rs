use crate::scene::spline::{DeformerScaleMode, SplineKind, SplineMeshDeformer};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct SplineEditorView {
    pub selected_point: Option<usize>,
    pub kind_idx: usize,
    pub show_preview: bool,
}

impl Default for SplineEditorView {
    fn default() -> Self { Self { selected_point: None, kind_idx: 1, show_preview: true } }
}

impl SplineEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, deformer: &mut SplineMeshDeformer) {
        let panel = Panel::new("Éditeur de Spline").with_icon(Icon::Spline);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("spline_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;
        let spline = &mut deformer.spline;

        let kind_lbl = SplineKind::ALL[self.kind_idx % SplineKind::ALL.len()].label();
        if Button::new(kind_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child("kind"), Rect::new(lx, y, 130.0, row_h)).clicked {
            self.kind_idx = (self.kind_idx + 1) % SplineKind::ALL.len();
            spline.kind = SplineKind::ALL[self.kind_idx % SplineKind::ALL.len()].clone();
        }
        let closed_lbl = if spline.closed { "Fermée" } else { "Ouverte" };
        if Button::new(closed_lbl).with_style(if spline.closed { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("closed"), Rect::new(lx + 136.0, y, 90.0, row_h)).clicked {
            spline.closed = !spline.closed;
        }
        y += row_h + sp;

        let mut res_f = spline.resolution as f64;
        Slider::new("Résolution", 4.0, 100.0).show(ui, id.child("res"), Rect::new(lx, y, w, row_h), &mut res_f);
        spline.resolution = res_f as usize;
        y += row_h + sp;

        let length = spline.length_approx();
        ui.draw_list.text(Vec2::new(lx, y), &format!("Longueur: {:.2}  |  Points: {}", length, spline.points.len()), p.text_muted, m.font_size_small);
        y += m.font_size_small + sp;

        ui.draw_list.text(Vec2::new(lx, y), "Points de contrôle", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_pt: Option<usize> = None;
        for (i, pt) in spline.points.iter_mut().enumerate() {
            let sel = self.selected_point == Some(i);
            let row = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(row, if sel { p.panel_active } else { p.panel }, 2.0);
            let lbl = format!("P{i}  ({:.2}, {:.2}, {:.2})", pt.position[0], pt.position[1], pt.position[2]);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &lbl, p.text, m.font_size_small);
            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 34.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("▼").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("psel{i}")), sel_r).clicked {
                self.selected_point = if sel { None } else { Some(i) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("pdel{i}")), del_r).clicked {
                remove_pt = Some(i);
            }
            y += row_h + 2.0;
            if sel {
                Slider::new("X", -50.0, 50.0).show(ui, id.child(&format!("px{i}")), Rect::new(lx, y, w, row_h), &mut pt.position[0]);
                y += row_h + sp;
                Slider::new("Y", -50.0, 50.0).show(ui, id.child(&format!("py{i}")), Rect::new(lx, y, w, row_h), &mut pt.position[1]);
                y += row_h + sp;
                Slider::new("Z", -50.0, 50.0).show(ui, id.child(&format!("pz{i}")), Rect::new(lx, y, w, row_h), &mut pt.position[2]);
                y += row_h + sp;
                Slider::new("Roll", -180.0, 180.0).show(ui, id.child(&format!("pr{i}")), Rect::new(lx, y, w, row_h), &mut pt.roll);
                y += row_h + sp;
                Slider::new("Scale", 0.1, 5.0).show(ui, id.child(&format!("ps{i}")), Rect::new(lx, y, w, row_h), &mut pt.scale);
                y += row_h + sp;
            }
        }
        if let Some(idx) = remove_pt {
            spline.remove_point(idx);
            if self.selected_point == Some(idx) { self.selected_point = None; }
        }
        if Button::new("+ Point").with_style(ButtonStyle::Secondary).show(ui, id.child("add_pt"), Rect::new(lx, y, w, row_h)).clicked {
            let last = spline.points.last().map(|pt| pt.position).unwrap_or([0.0; 3]);
            spline.add_point([last[0] + 1.0, last[1], last[2]]);
        }
        y += row_h + sp * 2.0;

        ui.draw_list.text(Vec2::new(lx, y), "Déformation de mesh", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;
        let mode_lbl = deformer.scale_mode.label();
        if Button::new(mode_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child("smode"), Rect::new(lx, y, 120.0, row_h)).clicked {
            deformer.scale_mode = if deformer.scale_mode == DeformerScaleMode::Stretch { DeformerScaleMode::Tile } else { DeformerScaleMode::Stretch };
        }
        let mut rep_f = deformer.repeat_count as f64;
        Slider::new("Répétitions", 1.0, 20.0).show(ui, id.child("rep"), Rect::new(lx + 126.0, y, w - 126.0, row_h), &mut rep_f);
        deformer.repeat_count = rep_f as usize;
    }
}

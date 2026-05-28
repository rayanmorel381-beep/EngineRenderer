use crate::scene::destruction::{DestructionBody, FractureMode};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct DestructionEditorView {
    pub mode_idx: usize,
}

impl Default for DestructionEditorView {
    fn default() -> Self { Self { mode_idx: 0 } }
}

impl DestructionEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, body: &mut DestructionBody) {
        let panel = Panel::new("Destruction").with_icon(Icon::Physics);
        panel.show_chrome(ui, rect);
        let body_rect = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("destr_ed");
        let lx = body_rect.x + pad;
        let w = body_rect.width - pad * 2.0;
        let mut y = body_rect.y + pad;

        let en_lbl = if body.enabled { "Activé" } else { "Désactivé" };
        if Button::new(en_lbl).with_style(if body.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(lx, y, 100.0, row_h)).clicked {
            body.enabled = !body.enabled;
        }
        let frac_lbl = if body.fractured { "Fracturé ✓" } else { "Intact" };
        let frac_col = if body.fractured { p.success } else { p.text_muted };
        ui.draw_list.text(Vec2::new(lx + 110.0, y + (row_h - m.font_size_small) * 0.5), frac_lbl, frac_col, m.font_size_small);
        y += row_h + sp;

        let mode_lbl = FractureMode::ALL[self.mode_idx % FractureMode::ALL.len()].label();
        if Button::new(mode_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child("mode"), Rect::new(lx, y, 160.0, row_h)).clicked {
            self.mode_idx = (self.mode_idx + 1) % FractureMode::ALL.len();
            body.mode = FractureMode::ALL[self.mode_idx % FractureMode::ALL.len()].clone();
        }
        y += row_h + sp;

        let mut chunks_f = body.chunk_count as f64;
        Slider::new("Nb morceaux", 2.0, 64.0).show(ui, id.child("chunks"), Rect::new(lx, y, w, row_h), &mut chunks_f);
        body.chunk_count = chunks_f as usize;
        y += row_h + sp;

        Slider::new("Santé", 0.0, 500.0).show(ui, id.child("health"), Rect::new(lx, y, w, row_h), &mut body.health);
        y += row_h + sp;
        Slider::new("Seuil impact", 1.0, 500.0).show(ui, id.child("thresh"), Rect::new(lx, y, w, row_h), &mut body.impact_threshold);
        y += row_h + sp;
        Slider::new("Durée débris (s)", 1.0, 60.0).show(ui, id.child("debris"), Rect::new(lx, y, w, row_h), &mut body.debris_lifetime);
        y += row_h + sp;

        if !body.fractured {
            if Button::new("Fracturer (sim)").with_style(ButtonStyle::Danger).show(ui, id.child("frac_btn"), Rect::new(lx, y, w, row_h)).clicked {
                body.fracture([0.0, 0.0, 0.0], [0.0, 10.0, 0.0], 42);
            }
        } else {
            let chunk_active = body.chunks.iter().filter(|c| c.active).count();
            let stats = format!("Actifs: {} / {}", chunk_active, body.chunks.len());
            ui.draw_list.text(Vec2::new(lx, y), &stats, p.text_muted, m.font_size_small);
            y += m.font_size_small + sp;
            if Button::new("Réinitialiser").with_style(ButtonStyle::Secondary).show(ui, id.child("reset_btn"), Rect::new(lx, y, w, row_h)).clicked {
                body.chunks.clear();
                body.fractured = false;
                body.health = body.impact_threshold * 2.0;
            }
        }
    }
}

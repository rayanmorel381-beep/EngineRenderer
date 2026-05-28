use crate::scene::softbody::SoftBody;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct SoftBodyEditorView {
    pub box_half: f64,
}

impl Default for SoftBodyEditorView {
    fn default() -> Self { Self { box_half: 0.5 } }
}

impl SoftBodyEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, sb: &mut Option<SoftBody>) {
        let panel = Panel::new("Corps mou (Soft Body)").with_icon(Icon::Mesh);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("softbody_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        Slider::new("Demi-taille boîte", 0.1, 2.0).show(ui, id.child("half"), Rect::new(lx, y, w, row_h), &mut self.box_half);
        y += row_h + sp;

        if let Some(s) = sb.as_mut() {
            Slider::new("Amortissement", 0.0, 0.2).show(ui, id.child("damp"), Rect::new(lx, y, w, row_h), &mut s.damping);
            y += row_h + sp;
            Slider::new("Pression", 0.0, 5.0).show(ui, id.child("press"), Rect::new(lx, y, w, row_h), &mut s.pressure);
            y += row_h + sp;
            Slider::new("Gravité Y", -20.0, 0.0).show(ui, id.child("gravy"), Rect::new(lx, y, w, row_h), &mut s.gravity[1]);
            y += row_h + sp;
            let mut sub_f = s.substeps as f64;
            if Slider::new("Substeps", 1.0, 20.0).show(ui, id.child("sub"), Rect::new(lx, y, w, row_h), &mut sub_f) {
                s.substeps = (sub_f as usize).max(1);
            }
            y += row_h + sp;
            let info = format!("Sommets: {} | Contraintes: {}", s.vertices.len(), s.constraints.len());
            ui.draw_list.text(Vec2::new(lx, y), &info, p.text_muted, m.font_size_small);
            y += m.font_size_small + sp;
            if Button::new("Recréer boîte").with_style(ButtonStyle::Secondary)
                .show(ui, id.child("regen"), Rect::new(lx, y, w, row_h)).clicked {
                *sb = Some(SoftBody::from_box(self.box_half));
            }
            y += row_h + sp;
            if Button::new("Supprimer").with_style(ButtonStyle::Danger)
                .show(ui, id.child("del"), Rect::new(lx, y, w, row_h)).clicked {
                *sb = None;
            }
        } else {
            ui.draw_list.text(Vec2::new(lx, y), "Aucun corps mou", p.text_muted, m.font_size_normal);
            y += m.font_size_normal + sp * 2.0;
            if Button::new("Créer boîte molle").with_style(ButtonStyle::Primary)
                .show(ui, id.child("create"), Rect::new(lx, y, w, row_h)).clicked {
                *sb = Some(SoftBody::from_box(self.box_half));
            }
        }
    }
}

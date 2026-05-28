use crate::scene::cloth::ClothMesh;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct ClothEditorView {
    pub rows: usize,
    pub cols: usize,
    pub cell_size: f64,
}

impl Default for ClothEditorView {
    fn default() -> Self { Self { rows: 10, cols: 10, cell_size: 0.2 } }
}

impl ClothEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, cloth: &mut Option<ClothMesh>) {
        let panel = Panel::new("Tissu (Cloth)").with_icon(Icon::Mesh);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("cloth_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let mut rows_f = self.rows as f64;
        if Slider::new("Rangées", 2.0, 40.0).show(ui, id.child("rows"), Rect::new(lx, y, w, row_h), &mut rows_f) {
            self.rows = rows_f as usize;
        }
        y += row_h + sp;
        let mut cols_f = self.cols as f64;
        if Slider::new("Colonnes", 2.0, 40.0).show(ui, id.child("cols"), Rect::new(lx, y, w, row_h), &mut cols_f) {
            self.cols = cols_f as usize;
        }
        y += row_h + sp;
        Slider::new("Taille cellule", 0.05, 1.0).show(ui, id.child("cs"), Rect::new(lx, y, w, row_h), &mut self.cell_size);
        y += row_h + sp;

        if let Some(c) = cloth.as_mut() {
            Slider::new("Gravité Y", -20.0, 0.0).show(ui, id.child("gravy"), Rect::new(lx, y, w, row_h), &mut c.gravity[1]);
            y += row_h + sp;
            Slider::new("Vent X", -5.0, 5.0).show(ui, id.child("windx"), Rect::new(lx, y, w, row_h), &mut c.wind[0]);
            y += row_h + sp;
            Slider::new("Vent Z", -5.0, 5.0).show(ui, id.child("windz"), Rect::new(lx, y, w, row_h), &mut c.wind[2]);
            y += row_h + sp;
            Slider::new("Amortissement", 0.0, 0.1).show(ui, id.child("damp"), Rect::new(lx, y, w, row_h), &mut c.damping);
            y += row_h + sp;
            let mut sub_f = c.substeps as f64;
            if Slider::new("Substeps", 1.0, 20.0).show(ui, id.child("sub"), Rect::new(lx, y, w, row_h), &mut sub_f) {
                c.substeps = (sub_f as usize).max(1);
            }
            y += row_h + sp;
            let info = format!("Sommets: {} | Contraintes dist: {} | Contraintes bend: {}",
                c.vertices.len(), c.distance_constraints.len(), c.bend_constraints.len());
            ui.draw_list.text(Vec2::new(lx, y), &info, p.text_muted, m.font_size_small);
            y += m.font_size_small + sp;

            if Button::new("Recréer grille").with_style(ButtonStyle::Secondary)
                .show(ui, id.child("regen"), Rect::new(lx, y, w, row_h)).clicked {
                *cloth = Some(ClothMesh::new_grid(self.rows, self.cols, self.cell_size));
            }
            y += row_h + sp;
            if Button::new("Supprimer tissu").with_style(ButtonStyle::Danger)
                .show(ui, id.child("del"), Rect::new(lx, y, w, row_h)).clicked {
                *cloth = None;
            }
        } else {
            ui.draw_list.text(Vec2::new(lx, y), "Aucun tissu", p.text_muted, m.font_size_normal);
            y += m.font_size_normal + sp * 2.0;
            if Button::new("Créer tissu").with_style(ButtonStyle::Primary)
                .show(ui, id.child("create"), Rect::new(lx, y, w, row_h)).clicked {
                *cloth = Some(ClothMesh::new_grid(self.rows, self.cols, self.cell_size));
            }
        }
    }
}

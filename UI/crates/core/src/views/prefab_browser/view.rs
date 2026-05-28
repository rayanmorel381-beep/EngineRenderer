use crate::scene::prefab::{PrefabInstance, PrefabLibrary};
use crate::scene::Scene;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle};

pub struct PrefabBrowserView {
    pub selected: Option<usize>,
    pub pending_instantiate: Option<usize>,
    pub pending_delete: Option<usize>,
}

impl Default for PrefabBrowserView {
    fn default() -> Self { Self { selected: None, pending_instantiate: None, pending_delete: None } }
}

impl PrefabBrowserView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, library: &PrefabLibrary) {
        let panel = Panel::new("Prefabs").with_icon(Icon::Folder);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("prefab_br");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), &format!("{} prefab(s)", library.len()), p.text_muted, m.font_size_normal);
        y += m.font_size_normal + sp;

        for (i, prefab) in library.prefabs.iter().enumerate() {
            let active = self.selected == Some(i);
            let row = Rect::new(lx, y, w, row_h);
            let bg = if active { p.selection } else if i % 2 == 0 { p.panel } else { p.panel_hover };
            ui.draw_list.rect(row, bg, 0.0);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_normal) * 0.5), &prefab.name, p.text, m.font_size_normal);

            let btn_w = 70.0;
            let inst_r = Rect::new(row.x + row.width - btn_w * 2.0 - sp * 2.0, y + 2.0, btn_w, row_h - 4.0);
            let del_r = Rect::new(row.x + row.width - btn_w - sp, y + 2.0, btn_w, row_h - 4.0);
            if Button::primary("Ajouter").show(ui, id.child(&format!("inst{i}")), inst_r).clicked {
                self.pending_instantiate = Some(i);
            }
            if Button::new("Suppr.").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("del{i}")), del_r).clicked {
                self.pending_delete = Some(i);
            }

            if Button::new("").show(ui, id.child(&format!("sel{i}")), Rect::new(lx, y, row.width - btn_w * 2.0 - sp * 3.0, row_h)).clicked {
                self.selected = Some(i);
            }
            y += row_h + 2.0;
        }

        if library.is_empty() {
            ui.draw_list.text(Vec2::new(lx, y + 10.0), "Aucun prefab. Sélectionnez un objet\net cliquez 'Sauver en prefab'.", p.text_disabled, m.font_size_normal);
        }
    }

    pub fn flush_pending(&mut self, library: &PrefabLibrary, scene: &mut Scene) {
        if let Some(idx) = self.pending_instantiate.take() {
            PrefabInstance::new(idx).instantiate(library, scene);
        }
    }
}

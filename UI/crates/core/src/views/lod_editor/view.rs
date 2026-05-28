use crate::scene::lod::{LodGroup, LodLevel};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct LodEditorView {
    pub pending_add: bool,
    pub pending_remove: Option<usize>,
}

impl Default for LodEditorView {
    fn default() -> Self { Self { pending_add: false, pending_remove: None } }
}

impl LodEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, lod: &mut LodGroup) {
        let panel = Panel::new("LOD Group").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("lod_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let en_lbl = if lod.enabled { "LOD: Activé" } else { "LOD: Désactivé" };
        if Button::new(en_lbl).with_style(if lod.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(lx, y, 140.0, row_h)).clicked { lod.enabled = !lod.enabled; }
        y += row_h + sp;

        let cf_lbl = if lod.fade_mode_cross_fade { "Cross-Fade: ON" } else { "Cross-Fade: OFF" };
        if Button::new(cf_lbl).with_style(if lod.fade_mode_cross_fade { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("cf"), Rect::new(lx, y, 150.0, row_h)).clicked { lod.fade_mode_cross_fade = !lod.fade_mode_cross_fade; }
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), &format!("{} niveaux LOD", lod.levels.len()), p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 4.0;

        let mut remove_idx: Option<usize> = None;
        for (i, level) in lod.levels.iter_mut().enumerate() {
            let row = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(row, p.panel, 2.0);
            let lbl = format!("LOD {i}");
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_normal) * 0.5), &lbl, p.text_muted, m.font_size_normal);
            let slider_rect = Rect::new(lx + 55.0, y, w - 55.0 - 36.0 - sp, row_h);
            Slider::new("H.écran", 0.0, 1.0).show(ui, id.child(&format!("lod{i}")), slider_rect, &mut level.screen_relative_height);
            let del_r = Rect::new(lx + w - 32.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("del{i}")), del_r).clicked {
                remove_idx = Some(i);
            }
            y += row_h + 2.0;
        }
        if let Some(idx) = remove_idx { lod.levels.remove(idx); }

        y += 4.0;
        if Button::new("+ Ajouter niveau").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("add"), Rect::new(lx, y, w, row_h)).clicked {
            let next_threshold = lod.levels.last().map(|l| l.screen_relative_height * 0.5).unwrap_or(0.05);
            lod.levels.push(LodLevel::new(next_threshold.max(0.01)));
        }
    }
}

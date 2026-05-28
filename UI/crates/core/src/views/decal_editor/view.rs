use crate::scene::decal::{Decal, DecalBlendMode, DecalLayer};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct DecalEditorView {
    pub selected_decal: Option<usize>,
    pub blend_mode_idx: usize,
}

impl Default for DecalEditorView {
    fn default() -> Self { Self { selected_decal: None, blend_mode_idx: 3 } }
}

impl DecalEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, layer: &mut DecalLayer) {
        let panel = Panel::new("Éditeur de Décals").with_icon(Icon::Texture);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("decal_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Décals ({}) dans la scène", layer.decals.len()), p.text_muted, m.font_size_small);
        y += m.font_size_small + sp;

        let mut remove_decal: Option<usize> = None;
        for (i, decal) in layer.decals.iter_mut().enumerate() {
            let sel = self.selected_decal == Some(i);
            let row = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(row, if sel { p.panel_active } else { p.panel }, 2.0);
            let info = format!("{} [{}] op:{:.2}", decal.name, decal.blend_mode.label(), decal.opacity);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &info, p.text, m.font_size_small);
            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 34.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("▼").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("dsel{i}")), sel_r).clicked {
                self.selected_decal = if sel { None } else { Some(i) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("ddel{i}")), del_r).clicked {
                remove_decal = Some(i);
            }
            y += row_h + 2.0;
            if sel {
                Slider::new("Opacité", 0.0, 1.0).show(ui, id.child(&format!("dop{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut decal.opacity);
                y += row_h + sp;
                Slider::new("Taille X", 0.1, 20.0).show(ui, id.child(&format!("dsx{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut decal.size[0]);
                y += row_h + sp;
                Slider::new("Taille Y", 0.1, 20.0).show(ui, id.child(&format!("dsy{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut decal.size[1]);
                y += row_h + sp;
                Slider::new("Taille Z", 0.1, 20.0).show(ui, id.child(&format!("dsz{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut decal.size[2]);
                y += row_h + sp;
                Slider::new("Fondu profondeur", 0.0, 1.0).show(ui, id.child(&format!("ddf{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut decal.depth_fade);
                y += row_h + sp;
                Slider::new("Fondu angle", 0.0, 90.0).show(ui, id.child(&format!("daf{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut decal.angle_fade);
                y += row_h + sp;
                let bm_lbl = DecalBlendMode::ALL[self.blend_mode_idx % DecalBlendMode::ALL.len()].label();
                if Button::new(bm_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("dbm{i}")), Rect::new(lx + pad, y, 150.0, row_h)).clicked {
                    self.blend_mode_idx = (self.blend_mode_idx + 1) % DecalBlendMode::ALL.len();
                    decal.blend_mode = DecalBlendMode::ALL[self.blend_mode_idx % DecalBlendMode::ALL.len()].clone();
                }
                y += row_h + sp;
                let en_lbl = if decal.enabled { "Actif" } else { "Inactif" };
                if Button::new(en_lbl).with_style(if decal.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child(&format!("den{i}")), Rect::new(lx + pad, y, 80.0, row_h)).clicked {
                    decal.enabled = !decal.enabled;
                }
                y += row_h + sp;
            }
        }
        if let Some(idx) = remove_decal { layer.remove(idx); if self.selected_decal == Some(idx) { self.selected_decal = None; } }
        if Button::new("+ Décal").with_style(ButtonStyle::Secondary).show(ui, id.child("add_decal"), Rect::new(lx, y, w, row_h)).clicked {
            layer.add(Decal::new(format!("Décal {}", layer.decals.len() + 1)));
        }
    }
}

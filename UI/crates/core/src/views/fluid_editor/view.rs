use crate::scene::fluid::FluidVolume;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct FluidEditorView {
    pub spawn_count: usize,
    pub spawn_spacing: f64,
}

impl Default for FluidEditorView {
    fn default() -> Self { Self { spawn_count: 64, spawn_spacing: 0.15 } }
}

impl FluidEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, fluid: &mut Option<FluidVolume>) {
        let panel = Panel::new("Simulation fluide (SPH)").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("fluid_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let mut sc_f = self.spawn_count as f64;
        if Slider::new("Nb particules", 8.0, 512.0).show(ui, id.child("sc"), Rect::new(lx, y, w, row_h), &mut sc_f) {
            self.spawn_count = sc_f as usize;
        }
        y += row_h + sp;
        Slider::new("Espacement", 0.05, 0.5).show(ui, id.child("spacing"), Rect::new(lx, y, w, row_h), &mut self.spawn_spacing);
        y += row_h + sp;

        if let Some(f) = fluid.as_mut() {
            Slider::new("Densité repos", 100.0, 2000.0).show(ui, id.child("rho"), Rect::new(lx, y, w, row_h), &mut f.params.rest_density);
            y += row_h + sp;
            Slider::new("Viscosité", 0.0, 1.0).show(ui, id.child("visc"), Rect::new(lx, y, w, row_h), &mut f.params.viscosity);
            y += row_h + sp;
            Slider::new("Gravité Y", -20.0, 0.0).show(ui, id.child("gravy"), Rect::new(lx, y, w, row_h), &mut f.params.gravity[1]);
            y += row_h + sp;

            let info = format!("Particules actives: {}", f.particles.len());
            ui.draw_list.text(Vec2::new(lx, y), &info, p.text_muted, m.font_size_small);
            y += m.font_size_small + sp;

            if Button::new("Spawn cube").with_style(ButtonStyle::Secondary)
                .show(ui, id.child("spawn"), Rect::new(lx, y, (w - sp) * 0.5, row_h)).clicked {
                f.spawn_cube([0.0, 0.5, 0.0], self.spawn_count, self.spawn_spacing);
            }
            if Button::new("Vider").with_style(ButtonStyle::Danger)
                .show(ui, id.child("clear"), Rect::new(lx + (w - sp) * 0.5 + sp, y, (w - sp) * 0.5, row_h)).clicked {
                f.particles.clear();
            }
            y += row_h + sp;
            if Button::new("Supprimer volume").with_style(ButtonStyle::Danger)
                .show(ui, id.child("del"), Rect::new(lx, y, w, row_h)).clicked {
                *fluid = None;
            }
        } else {
            ui.draw_list.text(Vec2::new(lx, y), "Aucun volume fluide", p.text_muted, m.font_size_normal);
            y += m.font_size_normal + sp * 2.0;
            if Button::new("Créer volume fluide").with_style(ButtonStyle::Primary)
                .show(ui, id.child("create"), Rect::new(lx, y, w, row_h)).clicked {
                let mut fv = FluidVolume::new();
                fv.spawn_cube([0.0, 0.5, 0.0], self.spawn_count, self.spawn_spacing);
                *fluid = Some(fv);
            }
        }
    }
}

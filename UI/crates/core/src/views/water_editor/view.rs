use crate::scene::water::{GerstnerWave, WaterBody};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct WaterEditorView {
    pub selected_wave: Option<usize>,
}

impl Default for WaterEditorView {
    fn default() -> Self { Self { selected_wave: None } }
}

impl WaterEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, water: &mut WaterBody) {
        let panel = Panel::new("Éditeur d'eau").with_icon(Icon::Water);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("water_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let en_lbl = if water.enabled { "Activé" } else { "Désactivé" };
        if Button::new(en_lbl).with_style(if water.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(lx, y, 100.0, row_h)).clicked {
            water.enabled = !water.enabled;
        }
        y += row_h + sp;

        Slider::new("Niveau d'eau", -20.0, 20.0).show(ui, id.child("wlvl"), Rect::new(lx, y, w, row_h), &mut water.water_level);
        y += row_h + sp;
        Slider::new("Profondeur", 1.0, 200.0).show(ui, id.child("depth"), Rect::new(lx, y, w, row_h), &mut water.depth);
        y += row_h + sp;
        Slider::new("Densité (kg/m³)", 800.0, 1200.0).show(ui, id.child("dens"), Rect::new(lx, y, w, row_h), &mut water.density);
        y += row_h + sp;
        Slider::new("Seuil écume", 0.0, 2.0).show(ui, id.child("foam"), Rect::new(lx, y, w, row_h), &mut water.foam_threshold);
        y += row_h + sp;
        Slider::new("Intensité caustiques", 0.0, 3.0).show(ui, id.child("caust"), Rect::new(lx, y, w, row_h), &mut water.caustic_intensity);
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), "Vagues de Gerstner", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_wave: Option<usize> = None;
        for (i, wave) in water.waves.iter_mut().enumerate() {
            let sel = self.selected_wave == Some(i);
            let row = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(row, if sel { p.panel_active } else { p.panel }, 2.0);
            let info = format!("Vague {i}  A:{:.2} λ:{:.1} v:{:.1}", wave.amplitude, wave.wavelength, wave.speed);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &info, p.text, m.font_size_small);
            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 34.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("▼").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("wsel{i}")), sel_r).clicked {
                self.selected_wave = if sel { None } else { Some(i) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("wdel{i}")), del_r).clicked {
                remove_wave = Some(i);
            }
            y += row_h + 2.0;
            if sel {
                Slider::new("Amplitude", 0.0, 5.0).show(ui, id.child(&format!("wa{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut wave.amplitude);
                y += row_h + sp;
                Slider::new("Longueur d'onde", 0.5, 50.0).show(ui, id.child(&format!("wl{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut wave.wavelength);
                y += row_h + sp;
                Slider::new("Vitesse", 0.1, 10.0).show(ui, id.child(&format!("wv{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut wave.speed);
                y += row_h + sp;
                Slider::new("Inclinaison", 0.0, 1.0).show(ui, id.child(&format!("wst{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut wave.steepness);
                y += row_h + sp;
                Slider::new("Dir X", -1.0, 1.0).show(ui, id.child(&format!("wdx{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut wave.direction[0]);
                y += row_h + sp;
                Slider::new("Dir Z", -1.0, 1.0).show(ui, id.child(&format!("wdz{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut wave.direction[1]);
                y += row_h + sp;
            }
        }
        if let Some(idx) = remove_wave { water.waves.remove(idx); if self.selected_wave == Some(idx) { self.selected_wave = None; } }
        if Button::new("+ Vague").with_style(ButtonStyle::Secondary).show(ui, id.child("add_wave"), Rect::new(lx, y, w, row_h)).clicked {
            water.waves.push(GerstnerWave::default());
        }
    }
}

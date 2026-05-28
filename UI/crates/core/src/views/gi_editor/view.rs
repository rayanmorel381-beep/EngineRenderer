use crate::scene::gi::{GiMode, GiSettings};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown, Slider};

pub struct GiEditorView {
    pub mode_idx: usize,
    pub selected_probe: Option<usize>,
}

impl Default for GiEditorView {
    fn default() -> Self { Self { mode_idx: 2, selected_probe: None } }
}

impl GiEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, gi: &mut GiSettings) {
        let panel = Panel::new("Illumination globale (GI)").with_icon(Icon::Light);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("gi_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let mode_labels: Vec<&str> = GiMode::ALL.iter().map(|m| m.label()).collect();
        if Dropdown::new("Mode GI", &mode_labels).show(ui, id.child("mode"), Rect::new(lx, y, w, row_h), &mut self.mode_idx) {
            gi.mode = GiMode::ALL[self.mode_idx].clone();
        }
        y += row_h + sp;

        let mut bounces_f = gi.bounces as f64;
        if Slider::new("Rebonds", 0.0, 8.0).show(ui, id.child("bounce"), Rect::new(lx, y, w, row_h), &mut bounces_f) {
            gi.bounces = bounces_f as usize;
        }
        y += row_h + sp;

        Slider::new("Intensité ciel", 0.0, 5.0).show(ui, id.child("sky"), Rect::new(lx, y, w, row_h), &mut gi.sky_light_intensity);
        y += row_h + sp;
        Slider::new("Émission ×", 0.0, 4.0).show(ui, id.child("emi"), Rect::new(lx, y, w, row_h), &mut gi.emission_intensity_scale);
        y += row_h + sp;

        let ao_lbl = if gi.ambient_occlusion { "AO: Activé" } else { "AO: Désactivé" };
        if Button::new(ao_lbl).with_style(if gi.ambient_occlusion { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("ao"), Rect::new(lx, y, 140.0, row_h)).clicked { gi.ambient_occlusion = !gi.ambient_occlusion; }
        y += row_h + sp;

        if gi.ambient_occlusion {
            Slider::new("Rayon AO", 0.01, 2.0).show(ui, id.child("aor"), Rect::new(lx, y, w, row_h), &mut gi.ao_radius);
            y += row_h + sp;
        }

        ui.draw_list.text(Vec2::new(lx, y), &format!("Sondes lumière ({})", gi.probes.len()), p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_probe: Option<usize> = None;
        for (i, probe) in gi.probes.iter_mut().enumerate() {
            let row = Rect::new(lx, y, w, row_h);
            let is_sel = self.selected_probe == Some(i);
            ui.draw_list.rect(row, if is_sel { p.selection } else { p.panel }, 2.0);
            let lbl = format!("{} — r:{:.1} ×{:.1}", probe.name, probe.radius, probe.intensity);
            ui.draw_list.text(Vec2::new(lx+4.0, y+(row_h-m.font_size_normal)*0.5), &lbl, p.text, m.font_size_normal);
            let sel_r = Rect::new(lx+w-68.0, y+2.0, 32.0, row_h-4.0);
            let del_r = Rect::new(lx+w-32.0, y+2.0, 32.0, row_h-4.0);
            if Button::new("✓").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("psel{i}")), sel_r).clicked {
                self.selected_probe = if is_sel { None } else { Some(i) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("pdel{i}")), del_r).clicked {
                remove_probe = Some(i);
            }
            y += row_h + 2.0;
            if is_sel {
                Slider::new("Rayon", 0.5, 50.0).show(ui, id.child(&format!("pr{i}")), Rect::new(lx+8.0, y, w-8.0, row_h), &mut probe.radius);
                y += row_h + 2.0;
                Slider::new("Intensité", 0.0, 4.0).show(ui, id.child(&format!("pi{i}")), Rect::new(lx+8.0, y, w-8.0, row_h), &mut probe.intensity);
                y += row_h + 2.0;
            }
        }
        if let Some(idx) = remove_probe { gi.probes.remove(idx); self.selected_probe = None; }

        if Button::new("+ Sonde").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("add_probe"), Rect::new(lx, y, w, row_h)).clicked {
            gi.add_probe(format!("Sonde_{}", gi.probes.len()), [0.0; 3]);
        }
    }
}

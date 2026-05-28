use crate::scene::post_process::{PostProcessVolume, ToneMappingMode};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::Rect;
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown, Slider};

pub struct PostProcessView {
    tone_sel: usize,
    active_section: usize,
}

impl Default for PostProcessView {
    fn default() -> Self { Self { tone_sel: 2, active_section: 0 } }
}

impl PostProcessView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, vol: &mut PostProcessVolume) {
        let panel = Panel::new("Post-Processing").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let id = WidgetId::hash_str("pp_ed");
        let mut y = body.y + pad;

        let sections = ["Bloom", "SSAO", "DOF", "Color", "Vignette", "Autres"];
        let btn_w = (w - sp * (sections.len() as f64 - 1.0)) / sections.len() as f64;
        for (i, sec) in sections.iter().enumerate() {
            let active = self.active_section == i;
            let r = Rect::new(lx + i as f64 * (btn_w + sp), y, btn_w, row_h);
            if Button::new(sec).with_style(if active { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                .show(ui, id.child(sec), r).clicked { self.active_section = i; }
        }
        y += row_h + sp;

        match self.active_section {
            0 => {
                let en = if vol.bloom.enabled { "ON" } else { "OFF" };
                if Button::new(&format!("Bloom: {en}")).with_style(if vol.bloom.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child("bl_en"), Rect::new(lx, y, 100.0, row_h)).clicked { vol.bloom.enabled = !vol.bloom.enabled; }
                y += row_h + sp;
                Slider::new("Threshold", 0.0, 5.0).show(ui, id.child("bl_thr"), Rect::new(lx, y, w, row_h), &mut vol.bloom.threshold); y += row_h + sp;
                Slider::new("Intensity", 0.0, 10.0).show(ui, id.child("bl_int"), Rect::new(lx, y, w, row_h), &mut vol.bloom.intensity); y += row_h + sp;
                Slider::new("Scatter", 0.0, 1.0).show(ui, id.child("bl_sc"), Rect::new(lx, y, w, row_h), &mut vol.bloom.scatter);
            }
            1 => {
                let en = if vol.ssao.enabled { "ON" } else { "OFF" };
                if Button::new(&format!("SSAO: {en}")).with_style(if vol.ssao.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child("ao_en"), Rect::new(lx, y, 100.0, row_h)).clicked { vol.ssao.enabled = !vol.ssao.enabled; }
                y += row_h + sp;
                Slider::new("Intensity", 0.0, 4.0).show(ui, id.child("ao_int"), Rect::new(lx, y, w, row_h), &mut vol.ssao.intensity); y += row_h + sp;
                Slider::new("Radius", 0.0, 2.0).show(ui, id.child("ao_rad"), Rect::new(lx, y, w, row_h), &mut vol.ssao.radius); y += row_h + sp;
                Slider::new("Bias", 0.0, 0.1).show(ui, id.child("ao_bias"), Rect::new(lx, y, w, row_h), &mut vol.ssao.bias);
            }
            2 => {
                let en = if vol.dof.enabled { "ON" } else { "OFF" };
                if Button::new(&format!("DOF: {en}")).with_style(if vol.dof.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child("dof_en"), Rect::new(lx, y, 100.0, row_h)).clicked { vol.dof.enabled = !vol.dof.enabled; }
                y += row_h + sp;
                Slider::new("Focus", 0.1, 200.0).show(ui, id.child("dof_f"), Rect::new(lx, y, w, row_h), &mut vol.dof.focus_distance); y += row_h + sp;
                Slider::new("Aperture", 1.0, 32.0).show(ui, id.child("dof_ap"), Rect::new(lx, y, w, row_h), &mut vol.dof.aperture); y += row_h + sp;
                Slider::new("Focal Length", 1.0, 300.0).show(ui, id.child("dof_fl"), Rect::new(lx, y, w, row_h), &mut vol.dof.focal_length);
            }
            3 => {
                let tone_opts: Vec<&str> = ToneMappingMode::ALL.iter().map(|t| t.label()).collect();
                self.tone_sel = ToneMappingMode::ALL.iter().position(|t| *t == vol.color_grading.tone_mapping).unwrap_or(2);
                if Dropdown::new("Tone Mapping", &tone_opts).show(ui, id.child("tm"), Rect::new(lx, y, w, row_h), &mut self.tone_sel) {
                    vol.color_grading.tone_mapping = ToneMappingMode::ALL[self.tone_sel].clone();
                }
                y += row_h + sp;
                Slider::new("Exposure", -10.0, 10.0).show(ui, id.child("cg_ex"), Rect::new(lx, y, w, row_h), &mut vol.color_grading.exposure); y += row_h + sp;
                Slider::new("Contrast", -100.0, 100.0).show(ui, id.child("cg_con"), Rect::new(lx, y, w, row_h), &mut vol.color_grading.contrast); y += row_h + sp;
                Slider::new("Saturation", -100.0, 100.0).show(ui, id.child("cg_sat"), Rect::new(lx, y, w, row_h), &mut vol.color_grading.saturation); y += row_h + sp;
                Slider::new("Hue Shift", -180.0, 180.0).show(ui, id.child("cg_hue"), Rect::new(lx, y, w, row_h), &mut vol.color_grading.hue_shift);
            }
            4 => {
                let en = if vol.vignette.enabled { "ON" } else { "OFF" };
                if Button::new(&format!("Vignette: {en}")).with_style(if vol.vignette.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child("vg_en"), Rect::new(lx, y, 120.0, row_h)).clicked { vol.vignette.enabled = !vol.vignette.enabled; }
                y += row_h + sp;
                Slider::new("Intensity", 0.0, 1.0).show(ui, id.child("vg_int"), Rect::new(lx, y, w, row_h), &mut vol.vignette.intensity); y += row_h + sp;
                Slider::new("Smoothness", 0.01, 1.0).show(ui, id.child("vg_sm"), Rect::new(lx, y, w, row_h), &mut vol.vignette.smoothness);
            }
            5 => {
                let mb = if vol.motion_blur_enabled { "ON" } else { "OFF" };
                if Button::new(&format!("Motion Blur: {mb}")).with_style(if vol.motion_blur_enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child("mb_en"), Rect::new(lx, y, 140.0, row_h)).clicked { vol.motion_blur_enabled = !vol.motion_blur_enabled; }
                y += row_h + sp;
                Slider::new("MB Intensity", 0.0, 1.0).show(ui, id.child("mb_int"), Rect::new(lx, y, w, row_h), &mut vol.motion_blur_intensity); y += row_h + sp;
                let ca = if vol.chromatic_aberration_enabled { "ON" } else { "OFF" };
                if Button::new(&format!("Chromatic Aber.: {ca}")).with_style(if vol.chromatic_aberration_enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                    .show(ui, id.child("ca_en"), Rect::new(lx, y, 170.0, row_h)).clicked { vol.chromatic_aberration_enabled = !vol.chromatic_aberration_enabled; }
                y += row_h + sp;
                Slider::new("CA Intensity", 0.0, 1.0).show(ui, id.child("ca_int"), Rect::new(lx, y, w, row_h), &mut vol.chromatic_aberration_intensity);
            }
            _ => {}
        }
        let _ = y;
    }
}

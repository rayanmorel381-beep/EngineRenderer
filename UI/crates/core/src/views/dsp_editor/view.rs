use crate::scene::dsp::DspChain;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct DspEditorView {}

impl Default for DspEditorView {
    fn default() -> Self { Self {} }
}

impl DspEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, dsp: &mut DspChain) {
        let panel = Panel::new("DSP Audio").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("dsp_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), "Égaliseur paramétrique", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;
        let eq_lbl = if dsp.eq.enabled { "EQ: ON" } else { "EQ: OFF" };
        if Button::new(eq_lbl).with_style(if dsp.eq.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("eq_en"), Rect::new(lx, y, 80.0, row_h)).clicked { dsp.eq.enabled = !dsp.eq.enabled; }
        y += row_h + sp;
        if dsp.eq.enabled {
            for (i, band) in dsp.eq.bands.iter_mut().enumerate() {
                let lbl_f = format!("{:.0}Hz", band.frequency);
                ui.draw_list.text(Vec2::new(lx, y), &lbl_f, p.text_muted, m.font_size_small);
                Slider::new("dB", -12.0, 12.0).show(ui, id.child(&format!("eq{i}")), Rect::new(lx + 50.0, y, w - 50.0, row_h), &mut band.gain_db);
                y += row_h + 2.0;
            }
            y += sp;
        }

        ui.draw_list.text(Vec2::new(lx, y), "Compresseur", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;
        let comp_lbl = if dsp.compressor.enabled { "Compresseur: ON" } else { "Compresseur: OFF" };
        if Button::new(comp_lbl).with_style(if dsp.compressor.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("comp_en"), Rect::new(lx, y, 160.0, row_h)).clicked { dsp.compressor.enabled = !dsp.compressor.enabled; }
        y += row_h + sp;
        if dsp.compressor.enabled {
            Slider::new("Seuil (dB)", -60.0, 0.0).show(ui, id.child("cthr"), Rect::new(lx, y, w, row_h), &mut dsp.compressor.threshold_db);
            y += row_h + 2.0;
            Slider::new("Ratio", 1.0, 20.0).show(ui, id.child("crat"), Rect::new(lx, y, w, row_h), &mut dsp.compressor.ratio);
            y += row_h + 2.0;
            Slider::new("Attaque (ms)", 0.1, 200.0).show(ui, id.child("catk"), Rect::new(lx, y, w, row_h), &mut dsp.compressor.attack_ms);
            y += row_h + 2.0;
            Slider::new("Relâche (ms)", 10.0, 2000.0).show(ui, id.child("crel"), Rect::new(lx, y, w, row_h), &mut dsp.compressor.release_ms);
            y += row_h + 2.0;
            Slider::new("Gain compens.", -6.0, 24.0).show(ui, id.child("cmg"), Rect::new(lx, y, w, row_h), &mut dsp.compressor.makeup_gain_db);
            y += row_h + sp;
        }

        let rev_lbl = if dsp.reverb.enabled { "Reverb: ON" } else { "Reverb: OFF" };
        if Button::new(rev_lbl).with_style(if dsp.reverb.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("rev_en"), Rect::new(lx, y, 120.0, row_h)).clicked { dsp.reverb.enabled = !dsp.reverb.enabled; }
        y += row_h + sp;
        if dsp.reverb.enabled {
            Slider::new("Taille salle", 0.0, 1.0).show(ui, id.child("rsize"), Rect::new(lx, y, w, row_h), &mut dsp.reverb.room_size);
            y += row_h + 2.0;
            Slider::new("Wet", 0.0, 1.0).show(ui, id.child("rwet"), Rect::new(lx, y, w, row_h), &mut dsp.reverb.wet_level);
            y += row_h + sp;
        }

        let del_lbl = if dsp.delay.enabled { "Delay: ON" } else { "Delay: OFF" };
        if Button::new(del_lbl).with_style(if dsp.delay.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("del_en"), Rect::new(lx, y, 120.0, row_h)).clicked { dsp.delay.enabled = !dsp.delay.enabled; }
        y += row_h + sp;
        if dsp.delay.enabled {
            Slider::new("Délai (ms)", 10.0, 2000.0).show(ui, id.child("dms"), Rect::new(lx, y, w, row_h), &mut dsp.delay.delay_ms);
            y += row_h + 2.0;
            Slider::new("Feedback", 0.0, 0.99).show(ui, id.child("dfb"), Rect::new(lx, y, w, row_h), &mut dsp.delay.feedback);
            y += row_h + 2.0;
            Slider::new("Wet", 0.0, 1.0).show(ui, id.child("dwet"), Rect::new(lx, y, w, row_h), &mut dsp.delay.wet_mix);
        }
    }
}

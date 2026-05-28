use crate::scene::audio::{AudioMixer, AudioRolloff, AudioSource};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown, Slider};

pub struct AudioMixerView {
    rolloff_sel: usize,
}

impl Default for AudioMixerView {
    fn default() -> Self { Self { rolloff_sel: 1 } }
}

impl AudioMixerView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, mixer: &mut AudioMixer, source: Option<&mut AudioSource>) {
        let panel = Panel::new("Audio").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("audio_ed");

        let (buses_rect, source_rect) = body.split_left((body.width * 0.35).clamp(100.0, 200.0));
        ui.draw_list.rect(buses_rect, p.panel, 0.0);

        let mut y = buses_rect.y + pad;
        ui.draw_list.text(Vec2::new(buses_rect.x + pad, y), "Mixer", p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 4.0;

        Slider::new("Master", 0.0, 1.0).show(ui, id.child("mast"), Rect::new(buses_rect.x + pad, y, buses_rect.width - pad * 2.0, row_h), &mut mixer.master_volume);
        y += row_h + sp;

        for (i, bus) in mixer.buses.iter_mut().enumerate() {
            let r = Rect::new(buses_rect.x + pad, y, buses_rect.width - pad * 2.0, row_h);
            Slider::new(&bus.name.clone(), 0.0, 1.0).show(ui, id.child(&format!("bus{i}")), r, &mut bus.volume);
            y += row_h + sp;
            let mute_lbl = if bus.muted { "M" } else { "  " };
            if Button::new(mute_lbl).with_style(if bus.muted { ButtonStyle::Secondary } else { ButtonStyle::Primary })
                .show(ui, id.child(&format!("mute{i}")), Rect::new(buses_rect.x + pad, y, 30.0, row_h * 0.7)).clicked {
                bus.muted = !bus.muted;
            }
            y += row_h * 0.7 + 2.0;
        }

        if let Some(src) = source {
            let er = Rect::new(source_rect.x + pad, source_rect.y + pad, source_rect.width - pad * 2.0, source_rect.height - pad * 2.0);
            let mut sy = er.y;
            ui.draw_list.text(Vec2::new(er.x, sy), "Source Audio", p.text_muted, m.font_size_normal);
            sy += m.font_size_normal + 4.0;

            Slider::new("Volume", 0.0, 1.0).show(ui, id.child("vol"), Rect::new(er.x, sy, er.width, row_h), &mut src.volume); sy += row_h + sp;
            Slider::new("Pitch", 0.1, 3.0).show(ui, id.child("pitch"), Rect::new(er.x, sy, er.width, row_h), &mut src.pitch); sy += row_h + sp;
            Slider::new("Min Dist", 0.0, 100.0).show(ui, id.child("dmin"), Rect::new(er.x, sy, er.width, row_h), &mut src.min_distance); sy += row_h + sp;
            Slider::new("Max Dist", 1.0, 1000.0).show(ui, id.child("dmax"), Rect::new(er.x, sy, er.width, row_h), &mut src.max_distance); sy += row_h + sp;
            Slider::new("Doppler", 0.0, 5.0).show(ui, id.child("dop"), Rect::new(er.x, sy, er.width, row_h), &mut src.doppler_level); sy += row_h + sp;
            Slider::new("Reverb Mix", 0.0, 1.0).show(ui, id.child("rev"), Rect::new(er.x, sy, er.width, row_h), &mut src.reverb_zone_mix); sy += row_h + sp;

            let roll_opts: Vec<&str> = AudioRolloff::ALL.iter().map(|r| r.label()).collect();
            self.rolloff_sel = AudioRolloff::ALL.iter().position(|r| *r == src.rolloff).unwrap_or(1);
            if Dropdown::new("Rolloff", &roll_opts).show(ui, id.child("roll"), Rect::new(er.x, sy, er.width, row_h), &mut self.rolloff_sel) {
                src.rolloff = AudioRolloff::ALL[self.rolloff_sel].clone();
            }
            sy += row_h + sp;

            let loop_lbl = if src.looping { "Loop: ON" } else { "Loop: OFF" };
            if Button::new(loop_lbl).with_style(if src.looping { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                .show(ui, id.child("lp"), Rect::new(er.x, sy, er.width * 0.48, row_h)).clicked { src.looping = !src.looping; }
            let sp3d_lbl = if src.spatial { "3D: ON" } else { "3D: OFF" };
            if Button::new(sp3d_lbl).with_style(if src.spatial { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                .show(ui, id.child("spa"), Rect::new(er.x + er.width * 0.52, sy, er.width * 0.48, row_h)).clicked { src.spatial = !src.spatial; }
        }
    }
}

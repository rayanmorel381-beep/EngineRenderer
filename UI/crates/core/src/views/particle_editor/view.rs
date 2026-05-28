use crate::scene::particles::{EmitterShape, ParticleEmitter, SimSpace};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown, Slider};

pub struct ParticleEditorView {
    shape_sel: usize,
    sim_space_sel: usize,
    pub pending_add: bool,
    pub pending_remove: Option<usize>,
    pub selected: usize,
}

impl Default for ParticleEditorView {
    fn default() -> Self {
        Self { shape_sel: 0, sim_space_sel: 0, pending_add: false, pending_remove: None, selected: 0 }
    }
}

impl ParticleEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, emitters: &mut Vec<ParticleEmitter>) {
        let panel = Panel::new("Particles").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;

        let (list_rect, editor_rect) = body.split_left((body.width * 0.28).clamp(90.0, 160.0));
        ui.draw_list.rect(list_rect, p.panel, 0.0);

        let add_r = Rect::new(list_rect.x + pad, list_rect.y + pad, list_rect.width - pad * 2.0, row_h);
        if Button::icon(Icon::Add).show(ui, WidgetId::hash_str("pe_add"), add_r).clicked {
            self.pending_add = true;
        }
        let list_body = Rect::new(list_rect.x, list_rect.y + pad + row_h + 4.0, list_rect.width, list_rect.height - pad - row_h - 4.0);
        for (i, em) in emitters.iter().enumerate() {
            let y = list_body.y + i as f64 * (row_h + 2.0);
            if y + row_h > list_body.y + list_body.height { break; }
            let bg = if self.selected == i { p.accent } else { p.background };
            ui.draw_list.rect(Rect::new(list_body.x, y, list_body.width, row_h), bg, 2.0);
            let tc = if em.enabled { p.text } else { p.text_muted };
            ui.draw_list.text(Vec2::new(list_body.x + pad, y + (row_h - m.font_size_normal) * 0.5), &em.name, tc, m.font_size_normal);
            if Button::new("").show(ui, WidgetId::hash_str("pe_sel_").combine(WidgetId(i as u64)), Rect::new(list_body.x, y, list_body.width, row_h)).clicked {
                self.selected = i;
            }
        }

        let er = Rect::new(editor_rect.x + pad, editor_rect.y + pad, editor_rect.width - pad * 2.0, editor_rect.height - pad * 2.0);
        let id = WidgetId::hash_str("pe_edit");

        if emitters.is_empty() {
            ui.draw_list.text(Vec2::new(er.x, er.y), "Aucun émetteur", p.text_muted, m.font_size_normal);
            return;
        }
        let Some(em) = emitters.get_mut(self.selected) else { return };
        let mut y = er.y;

        let en_label = if em.enabled { "Activé" } else { "Désactivé" };
        if Button::new(en_label).with_style(if em.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(er.x, y, 80.0, row_h)).clicked { em.enabled = !em.enabled; }
        if Button::new("Supprimer").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("rm"), Rect::new(er.x + 84.0, y, 90.0, row_h)).clicked {
            self.pending_remove = Some(self.selected);
        }
        y += row_h + sp;

        let shape_opts: Vec<&str> = EmitterShape::ALL.iter().map(|s| s.label()).collect();
        self.shape_sel = EmitterShape::ALL.iter().position(|s| *s == em.shape).unwrap_or(0);
        if Dropdown::new("Shape", &shape_opts).show(ui, id.child("sh"), Rect::new(er.x, y, er.width, row_h), &mut self.shape_sel) {
            em.shape = EmitterShape::ALL[self.shape_sel].clone();
        }
        y += row_h + sp;

        let ss_opts: Vec<&str> = SimSpace::ALL.iter().map(|s| s.label()).collect();
        self.sim_space_sel = SimSpace::ALL.iter().position(|s| *s == em.sim_space).unwrap_or(0);
        if Dropdown::new("Sim Space", &ss_opts).show(ui, id.child("ss"), Rect::new(er.x, y, er.width, row_h), &mut self.sim_space_sel) {
            em.sim_space = SimSpace::ALL[self.sim_space_sel].clone();
        }
        y += row_h + sp;

        let mut max_f = em.max_particles as f64;
        if Slider::new("Max Particles", 1.0, 10000.0).step(1.0).show(ui, id.child("maxp"), Rect::new(er.x, y, er.width, row_h), &mut max_f) {
            em.max_particles = max_f as usize;
        }
        y += row_h + sp;

        Slider::new("Emission Rate", 0.0, 1000.0).show(ui, id.child("er"), Rect::new(er.x, y, er.width, row_h), &mut em.emission_rate);
        y += row_h + sp;
        Slider::new("Life Min", 0.1, 20.0).show(ui, id.child("lmin"), Rect::new(er.x, y, er.width, row_h), &mut em.lifetime_min);
        y += row_h + sp;
        Slider::new("Life Max", 0.1, 20.0).show(ui, id.child("lmax"), Rect::new(er.x, y, er.width, row_h), &mut em.lifetime_max);
        y += row_h + sp;
        Slider::new("Speed Min", 0.0, 50.0).show(ui, id.child("smin"), Rect::new(er.x, y, er.width, row_h), &mut em.start_speed_min);
        y += row_h + sp;
        Slider::new("Speed Max", 0.0, 50.0).show(ui, id.child("smax"), Rect::new(er.x, y, er.width, row_h), &mut em.start_speed_max);
        y += row_h + sp;
        Slider::new("Size Min", 0.001, 5.0).show(ui, id.child("szmin"), Rect::new(er.x, y, er.width, row_h), &mut em.start_size_min);
        y += row_h + sp;
        Slider::new("Size Max", 0.001, 5.0).show(ui, id.child("szmax"), Rect::new(er.x, y, er.width, row_h), &mut em.start_size_max);
        y += row_h + sp;
        Slider::new("Gravity", -3.0, 3.0).show(ui, id.child("grav"), Rect::new(er.x, y, er.width, row_h), &mut em.gravity_scale);
        y += row_h + sp;
        Slider::new("Cone Angle", 0.0, 90.0).show(ui, id.child("cone"), Rect::new(er.x, y, er.width, row_h), &mut em.shape_angle);
        y += row_h + sp;
        Slider::new("Radius", 0.0, 20.0).show(ui, id.child("rad"), Rect::new(er.x, y, er.width, row_h), &mut em.shape_radius);
        y += row_h + sp;
        Slider::new("Duration", 0.1, 60.0).show(ui, id.child("dur"), Rect::new(er.x, y, er.width, row_h), &mut em.duration);
        y += row_h + sp;

        let lp_label = if em.looping { "Loop: ON" } else { "Loop: OFF" };
        if Button::new(lp_label).with_style(if em.looping { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("lp"), Rect::new(er.x, y, er.width * 0.48, row_h)).clicked { em.looping = !em.looping; }
        let sol_label = if em.size_over_lifetime { "Size OL: ON" } else { "Size OL: OFF" };
        if Button::new(sol_label).with_style(if em.size_over_lifetime { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("sol"), Rect::new(er.x + er.width * 0.52, y, er.width * 0.48, row_h)).clicked { em.size_over_lifetime = !em.size_over_lifetime; }
        y += row_h + sp;

        let col_label = if em.color_over_lifetime { "Color OL: ON" } else { "Color OL: OFF" };
        if Button::new(col_label).with_style(if em.color_over_lifetime { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("col"), Rect::new(er.x, y, er.width, row_h)).clicked { em.color_over_lifetime = !em.color_over_lifetime; }

        let cnt = em.particle_count();
        ui.draw_list.text(Vec2::new(er.x, y + row_h + sp), &format!("{}/{} particules actives", cnt, em.max_particles), p.text_muted, m.font_size_normal);
    }
}

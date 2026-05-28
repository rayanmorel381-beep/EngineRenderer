use crate::scene::ai_perception::AiPerceptionComponent;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct AiPerceptionEditorView {}

impl Default for AiPerceptionEditorView {
    fn default() -> Self { Self {} }
}

impl AiPerceptionEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, comp: &mut AiPerceptionComponent) {
        let panel = Panel::new("Perception IA").with_icon(Icon::Eye);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("ai_perc_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let en_lbl = if comp.config.enabled { "Activé" } else { "Désactivé" };
        if Button::new(en_lbl).with_style(if comp.config.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(lx, y, 100.0, row_h)).clicked {
            comp.config.enabled = !comp.config.enabled;
        }
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), "Vue", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let sight_enabled = comp.config.sight.is_some();
        let sight_lbl = if sight_enabled { "Vue: ON" } else { "Vue: OFF" };
        if Button::new(sight_lbl).with_style(if sight_enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("sight_en"), Rect::new(lx, y, 90.0, row_h)).clicked {
            if sight_enabled { comp.config.sight = None; } else { comp.config.sight = Some(crate::scene::ai_perception::SightConfig::default()); }
        }
        y += row_h + sp;

        if let Some(sight) = &mut comp.config.sight {
            Slider::new("Rayon vue", 1.0, 200.0).show(ui, id.child("sr"), Rect::new(lx + pad, y, w - pad, row_h), &mut sight.radius);
            y += row_h + sp;
            Slider::new("Rayon perte", 1.0, 300.0).show(ui, id.child("slr"), Rect::new(lx + pad, y, w - pad, row_h), &mut sight.lose_sight_radius);
            y += row_h + sp;
            Slider::new("Angle périphérique", 10.0, 180.0).show(ui, id.child("spa"), Rect::new(lx + pad, y, w - pad, row_h), &mut sight.peripheral_angle);
            y += row_h + sp;
            Slider::new("Succès auto (dist)", 0.5, 20.0).show(ui, id.child("sar"), Rect::new(lx + pad, y, w - pad, row_h), &mut sight.auto_success_range);
            y += row_h + sp;
        }

        ui.draw_list.text(Vec2::new(lx, y), "Ouïe", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let hear_enabled = comp.config.hearing.is_some();
        let hear_lbl = if hear_enabled { "Ouïe: ON" } else { "Ouïe: OFF" };
        if Button::new(hear_lbl).with_style(if hear_enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("hear_en"), Rect::new(lx, y, 90.0, row_h)).clicked {
            if hear_enabled { comp.config.hearing = None; } else { comp.config.hearing = Some(crate::scene::ai_perception::HearingConfig::default()); }
        }
        y += row_h + sp;

        if let Some(hearing) = &mut comp.config.hearing {
            Slider::new("Rayon ouïe", 1.0, 100.0).show(ui, id.child("hr"), Rect::new(lx + pad, y, w - pad, row_h), &mut hearing.radius);
            y += row_h + sp;
            Slider::new("Seuil volume", 0.0, 1.0).show(ui, id.child("hvt"), Rect::new(lx + pad, y, w - pad, row_h), &mut hearing.volume_threshold);
            y += row_h + sp;
        }

        Slider::new("Oubli (s)", 1.0, 30.0).show(ui, id.child("forget"), Rect::new(lx, y, w, row_h), &mut comp.config.forget_time);
        y += row_h + sp;

        let mut max_f = comp.config.max_perceived as f64;
        Slider::new("Perçus max", 1.0, 32.0).show(ui, id.child("maxp"), Rect::new(lx, y, w, row_h), &mut max_f);
        comp.config.max_perceived = max_f as usize;
        y += row_h + sp;

        let count = comp.perceived.iter().filter(|a| a.currently_perceived).count();
        ui.draw_list.text(Vec2::new(lx, y), &format!("Acteurs perçus: {}", count), p.text_muted, m.font_size_small);
        y += m.font_size_small + sp;

        for actor in &comp.perceived {
            let col = if actor.currently_perceived { p.success } else { p.text_muted };
            let lbl = format!("ID:{:?}  pos:({:.1},{:.1},{:.1})  [{}]", actor.target, actor.last_known_position[0], actor.last_known_position[1], actor.last_known_position[2], actor.sense.label());
            ui.draw_list.text(Vec2::new(lx + pad, y), &lbl, col, m.font_size_small);
            y += m.font_size_small + 3.0;
        }
    }
}

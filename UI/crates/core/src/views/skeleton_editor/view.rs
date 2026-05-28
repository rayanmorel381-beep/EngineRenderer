use crate::scene::skeleton::Skeleton;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle};

pub struct SkeletonEditorView {
    pub selected_state: usize,
    pub selected_bone: usize,
}

impl Default for SkeletonEditorView {
    fn default() -> Self { Self { selected_state: 0, selected_bone: 0 } }
}

impl SkeletonEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, skeleton: &mut Skeleton) {
        let panel = Panel::new("Skeleton / Animation SM").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;

        let (left, right) = body.split_left((body.width * 0.4).clamp(100.0, 220.0));
        let id = WidgetId::hash_str("skel_ed");

        ui.draw_list.rect(left, p.panel, 0.0);
        ui.draw_list.text(Vec2::new(left.x + pad, left.y + pad), "Bones", p.text_muted, m.font_size_normal);
        let mut by = left.y + pad + m.font_size_normal + 4.0;
        for (i, bone) in skeleton.bones.iter().enumerate() {
            let r = Rect::new(left.x + pad, by, left.width - pad * 2.0, row_h);
            let bg = if self.selected_bone == i { p.accent } else { p.background };
            ui.draw_list.rect(r, bg, 2.0);
            let indent = bone.parent.map(|_| 12.0).unwrap_or(0.0);
            ui.draw_list.text(Vec2::new(r.x + indent, by + (row_h - m.font_size_normal) * 0.5), &bone.name, p.text, m.font_size_normal);
            if Button::new("").show(ui, id.child(&format!("b{i}")), r).clicked {
                self.selected_bone = i;
            }
            by += row_h + 2.0;
        }

        let er = Rect::new(right.x + pad, right.y + pad, right.width - pad * 2.0, right.height - pad * 2.0);
        let mut y = er.y;

        ui.draw_list.text(Vec2::new(er.x, y), "State Machine", p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 4.0;

        let sm = &mut skeleton.state_machine;
        for (i, state) in sm.states.iter().enumerate() {
            let active = sm.current_state == i;
            let bg = if active { p.accent } else { p.panel_active };
            let r = Rect::new(er.x, y, er.width, row_h);
            ui.draw_list.rect(r, bg, 3.0);
            ui.draw_list.text(Vec2::new(er.x + pad, y + (row_h - m.font_size_normal) * 0.5), state, p.text, m.font_size_normal);
            if Button::new("").show(ui, id.child(&format!("st{i}")), r).clicked {
                self.selected_state = i;
            }
            y += row_h + 2.0;
        }
        y += sp;

        let play_lbl = if skeleton.playing { "Pause" } else { "Play" };
        if Button::new(play_lbl).with_style(ButtonStyle::Primary)
            .show(ui, id.child("play"), Rect::new(er.x, y, 60.0, row_h)).clicked {
            skeleton.playing = !skeleton.playing;
        }
        if Button::new("Stop").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("stop"), Rect::new(er.x + 64.0, y, 60.0, row_h)).clicked {
            skeleton.playing = false;
            skeleton.current_time = 0.0;
        }
        y += row_h + sp;

        let state_count = sm.states.len();
        if self.selected_state < state_count {
            let cur = &sm.states[self.selected_state];
            ui.draw_list.text(Vec2::new(er.x, y), &format!("Etat: {cur}"), p.text, m.font_size_normal);
            y += m.font_size_normal + 4.0;
            if Button::new("Transition rapide (0.3s)").with_style(ButtonStyle::Secondary)
                .show(ui, id.child("trans"), Rect::new(er.x, y, er.width, row_h)).clicked {
                let target = (self.selected_state + 1) % state_count;
                skeleton.state_machine.trigger_transition_to(target, 0.3);
            }
        }
    }
}

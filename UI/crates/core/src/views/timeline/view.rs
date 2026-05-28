use crate::scene::animation::{AnimTrack, AnimationClip, Animator, Easing, Keyframe};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, Slider, Tab, TabBar};

#[derive(Clone, Debug)]
pub struct TimelineTrack {
    pub name: String,
    pub keyframes: Vec<f64>,
}

impl TimelineTrack {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), keyframes: Vec::new() }
    }
}

pub struct TimelineView {
    pub tracks: Vec<TimelineTrack>,
    pub current_time: f64,
    pub duration: f64,
    pub playing: bool,
    active_tab: usize,
    clip_name_buf: String,
    clip_duration_buf: f64,
    selected_track: Option<usize>,
    kf_time_buf: f64,
    kf_value_buf: f64,
    kf_easing: Easing,
}

impl Default for TimelineView {
    fn default() -> Self {
        Self {
            tracks: Vec::new(),
            current_time: 0.0,
            duration: 10.0,
            playing: false,
            active_tab: 0,
            clip_name_buf: String::from("Clip"),
            clip_duration_buf: 5.0,
            selected_track: None,
            kf_time_buf: 0.0,
            kf_value_buf: 0.0,
            kf_easing: Easing::Linear,
        }
    }
}

impl TimelineView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Timeline").with_icon(Icon::Play);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let tab_h = ui.theme.metrics.tab_height;
        let tabs = [Tab::new("Séquenceur"), Tab::new("Éditeur de clip")];
        let (tab_rect, body_rect) = body.split_top(tab_h);
        TabBar::new(&tabs).show(ui, WidgetId::hash_str("timeline_tabs"), tab_rect, &mut self.active_tab);
        match self.active_tab {
            0 => self.show_sequencer(ui, body_rect),
            _ => self.show_clip_editor(ui, body_rect),
        }
    }

    fn show_sequencer(&mut self, ui: &mut UiContext, rect: Rect) {
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let header_h = m.row_height;
        let (ctrl_rect, track_area) = rect.split_top(header_h);
        let btn_w = 28.0;
        let pad = 4.0;

        let rew_r  = Rect::new(ctrl_rect.x + pad, ctrl_rect.y + 2.0, btn_w, header_h - 4.0);
        let play_r = Rect::new(rew_r.x + btn_w + 2.0, rew_r.y, btn_w, rew_r.height);
        let stop_r = Rect::new(play_r.x + btn_w + 2.0, rew_r.y, btn_w, rew_r.height);

        if Button::icon(Icon::ChevronLeft).show(ui, WidgetId::hash_str("tl_rew"), rew_r).clicked {
            self.current_time = 0.0;
        }
        let play_icon = if self.playing { Icon::Pause } else { Icon::Play };
        if Button::icon(play_icon).show(ui, WidgetId::hash_str("tl_play"), play_r).clicked {
            self.playing = !self.playing;
        }
        if Button::icon(Icon::Stop).show(ui, WidgetId::hash_str("tl_stop"), stop_r).clicked {
            self.playing = false;
            self.current_time = 0.0;
        }

        let t_label = format!("{:.3}s / {:.3}s", self.current_time, self.duration);
        ui.draw_list.text(
            Vec2::new(stop_r.x + btn_w + pad * 2.0, ctrl_rect.y + (header_h - m.font_size_normal) * 0.5),
            t_label, p.text, m.font_size_normal,
        );

        if self.tracks.is_empty() {
            ui.draw_list.text(
                Vec2::new(track_area.x + pad, track_area.y + pad),
                "Aucune track — ajoutez des keyframes via l'éditeur de clip",
                p.text_muted, m.font_size_normal,
            );
            return;
        }

        let label_w = (track_area.width * 0.22).clamp(80.0, 150.0);
        let track_h = m.row_height;
        ui.draw_list.push_clip(track_area);
        for (i, track) in self.tracks.iter().enumerate() {
            let y = track_area.y + i as f64 * track_h;
            if y + track_h > track_area.y + track_area.height { break; }
            let row_col = if i % 2 == 0 { p.panel } else { p.background };
            ui.draw_list.rect(Rect::new(track_area.x, y, track_area.width, track_h), row_col, 0.0);
            ui.draw_list.text(
                Vec2::new(track_area.x + pad, y + (track_h - m.font_size_normal) * 0.5),
                &track.name, p.text_muted, m.font_size_normal,
            );
            let lane = Rect::new(track_area.x + label_w, y + 2.0, (track_area.width - label_w).max(0.0), track_h - 4.0);
            for kf in &track.keyframes {
                let ratio = (kf / self.duration.max(f64::EPSILON)).clamp(0.0, 1.0);
                let x = lane.x + ratio * lane.width;
                ui.draw_list.rect(Rect::new(x - 4.0, lane.y, 8.0, lane.height), p.accent, 3.0);
            }
        }
        let cursor_ratio = (self.current_time / self.duration.max(f64::EPSILON)).clamp(0.0, 1.0);
        let cx = track_area.x + label_w + cursor_ratio * (track_area.width - label_w).max(0.0);
        ui.draw_list.line(Vec2::new(cx, track_area.y), Vec2::new(cx, track_area.y + track_area.height), p.accent, 1.5);
        ui.draw_list.pop_clip();
    }

    fn show_clip_editor(&mut self, ui: &mut UiContext, rect: Rect) {
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let mut y = rect.y + pad;

        ui.draw_list.text(Vec2::new(rect.x + pad, y), "Durée du clip (s)", p.text, m.font_size_normal);
        y += row_h;
        Slider::new("Durée", 0.1, 120.0).show(
            ui, WidgetId::hash_str("clip_dur"),
            Rect::new(rect.x + pad, y, rect.width - pad * 2.0, row_h),
            &mut self.clip_duration_buf,
        );
        y += row_h + 4.0;

        if Button::new("Réinitialiser les tracks").show(ui, WidgetId::hash_str("clip_reset"), Rect::new(rect.x + pad, y, rect.width - pad * 2.0, row_h)).clicked {
            self.tracks.clear();
            self.selected_track = None;
        }
        y += row_h + 8.0;

        ui.draw_list.text(Vec2::new(rect.x + pad, y), "Ajouter une track", p.text, m.font_size_normal);
        y += row_h;
        let props = ["pos.x", "pos.y", "pos.z", "rot.x", "rot.y", "rot.z", "scl.x", "scl.y", "scl.z"];
        let col_w = (rect.width - pad * 2.0) / props.len() as f64;
        for (i, prop) in props.iter().enumerate() {
            let r = Rect::new(rect.x + pad + i as f64 * col_w, y, col_w - 2.0, row_h);
            let id = WidgetId::hash_str("track_add_").combine(WidgetId::hash_str(prop));
            if Button::new(*prop).show(ui, id, r).clicked {
                if !self.tracks.iter().any(|t| t.name == *prop) {
                    self.tracks.push(TimelineTrack::new(*prop));
                }
            }
        }
        y += row_h + 8.0;

        ui.draw_list.text(Vec2::new(rect.x + pad, y), "Nouveau keyframe", p.text, m.font_size_normal);
        y += row_h;
        let half = (rect.width - pad * 2.0) * 0.47;
        Slider::new("Temps (s)", 0.0, self.clip_duration_buf).show(
            ui, WidgetId::hash_str("kf_time"),
            Rect::new(rect.x + pad, y, half, row_h),
            &mut self.kf_time_buf,
        );
        Slider::new("Valeur", -100.0, 100.0).show(
            ui, WidgetId::hash_str("kf_val"),
            Rect::new(rect.x + pad + half + pad, y, half, row_h),
            &mut self.kf_value_buf,
        );
        y += row_h + 4.0;

        let easing_entries: [(&str, Easing); 5] = [
            ("Lin", Easing::Linear), ("In", Easing::EaseIn),
            ("Out", Easing::EaseOut), ("InOut", Easing::EaseInOut), ("Step", Easing::Step),
        ];
        let ew = (rect.width - pad * 2.0) / easing_entries.len() as f64;
        for (i, (label, easing)) in easing_entries.iter().enumerate() {
            let r = Rect::new(rect.x + pad + i as f64 * ew, y, ew - 2.0, row_h);
            let id = WidgetId::hash_str("ease_").combine(WidgetId::hash_str(label));
            let style = if self.kf_easing == *easing {
                crate::ui::widgets::ButtonStyle::Primary
            } else {
                crate::ui::widgets::ButtonStyle::Secondary
            };
            if Button::new(label).with_style(style).show(ui, id, r).clicked {
                self.kf_easing = *easing;
            }
        }
        y += row_h + 4.0;

        if let Some(ti) = self.selected_track {
            let add_r = Rect::new(rect.x + pad, y, rect.width - pad * 2.0, row_h);
            if Button::new("+ Ajouter keyframe").show(ui, WidgetId::hash_str("kf_add"), add_r).clicked {
                if let Some(track) = self.tracks.get_mut(ti) {
                    track.keyframes.push(self.kf_time_buf);
                    track.keyframes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
                }
            }
            y += row_h + 4.0;
        }

        let available_h = (rect.y + rect.height - y - pad).max(0.0);
        let track_h = m.row_height;
        ui.draw_list.push_clip(Rect::new(rect.x, y, rect.width, available_h));
        for (i, track) in self.tracks.iter().enumerate() {
            let ty = y + i as f64 * track_h;
            if ty + track_h > rect.y + rect.height { break; }
            let active = self.selected_track == Some(i);
            let col = if active { p.accent } else { p.panel_active };
            ui.draw_list.rect(Rect::new(rect.x + pad, ty, rect.width - pad * 2.0, track_h - 2.0), col, 2.0);
            let txt_col = if active { p.text } else { p.text_muted };
            ui.draw_list.text(
                Vec2::new(rect.x + pad * 2.0, ty + (track_h - m.font_size_normal) * 0.5),
                format!("{} ({} kf)", &track.name, track.keyframes.len()),
                txt_col, m.font_size_normal,
            );
            let id = WidgetId::hash_str("tsel_").combine(WidgetId::hash_str(&track.name));
            if Button::new("").show(ui, id, Rect::new(rect.x + pad, ty, rect.width - pad * 2.0, track_h - 2.0)).clicked {
                self.selected_track = Some(i);
            }
        }
        ui.draw_list.pop_clip();
    }

    pub fn advance(&mut self, dt: f64) {
        if self.playing {
            self.current_time += dt;
            if self.current_time > self.duration {
                self.current_time = 0.0;
                self.playing = false;
            }
        }
    }

    pub fn build_animator(&self) -> Animator {
        let mut clip = AnimationClip::new(&self.clip_name_buf, self.clip_duration_buf);
        for track in &self.tracks {
            let mut anim_track = AnimTrack::new(&track.name);
            for &t in &track.keyframes {
                anim_track.add_keyframe(Keyframe::new(t, self.kf_value_buf));
            }
            clip.add_track(anim_track);
        }
        let mut animator = Animator::new();
        animator.clips.push(clip);
        animator
    }
}


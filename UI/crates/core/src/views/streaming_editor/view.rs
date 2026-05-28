use crate::scene::streaming::{AssetStreamingConfig, StreamState};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct StreamingEditorView {}

impl Default for StreamingEditorView {
    fn default() -> Self { Self {} }
}

impl StreamingEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, config: &mut AssetStreamingConfig) {
        let panel = Panel::new("Streaming assets").with_icon(Icon::Folder);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("streaming_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let en_lbl = if config.enabled { "Streaming: Activé" } else { "Streaming: Désactivé" };
        if Button::new(en_lbl).with_style(if config.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(lx, y, 180.0, row_h)).clicked { config.enabled = !config.enabled; }
        y += row_h + sp;

        Slider::new("Mémoire max (Mo)", 256.0, 8192.0).show(ui, id.child("mem"), Rect::new(lx, y, w, row_h), &mut config.budget.max_memory_mb);
        y += row_h + sp;
        Slider::new("Intervalle tick (s)", 0.1, 5.0).show(ui, id.child("tick"), Rect::new(lx, y, w, row_h), &mut config.tick_interval_s);
        y += row_h + sp;

        let dist_lbl = if config.distance_based { "Dist. based: ON" } else { "Dist. based: OFF" };
        if Button::new(dist_lbl).with_style(if config.distance_based { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("dist"), Rect::new(lx, y, 160.0, row_h)).clicked { config.distance_based = !config.distance_based; }
        y += row_h + sp;

        let stats = format!("Chargés: {} | En cours: {} | Total: {} | Mémoire: {:.0}/{:.0} Mo",
            config.loaded_count(), config.loading_count(), config.levels.len(),
            config.budget.used_memory_mb, config.budget.max_memory_mb);
        ui.draw_list.text(Vec2::new(lx, y), &stats, p.text_muted, m.font_size_small);
        y += m.font_size_small + sp;

        ui.draw_list.text(Vec2::new(lx, y), "Niveaux", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_level: Option<usize> = None;
        for (i, level) in config.levels.iter_mut().enumerate() {
            let row = Rect::new(lx, y, w, row_h);
            let state_col = match level.state {
                StreamState::Loaded => p.success,
                StreamState::Loading | StreamState::Requested => p.warning,
                StreamState::Unloaded | StreamState::Unloading => p.text_muted,
            };
            ui.draw_list.rect(row, p.panel, 2.0);
            let lbl = format!("{} [{}]", level.name, level.state.label());
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &lbl, state_col, m.font_size_small);
            let del_r = Rect::new(lx + w - 32.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("ldel{i}")), del_r).clicked {
                remove_level = Some(i);
            }
            y += row_h + 2.0;
        }
        if let Some(idx) = remove_level { config.levels.remove(idx); }

        if Button::new("+ Niveau").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("add_level"), Rect::new(lx, y, w, row_h)).clicked {
            let name = format!("Level_{}", config.levels.len());
            config.levels.push(crate::scene::streaming::StreamingLevel::new(name.clone(), format!("levels/{}.bin", name)));
        }
    }
}

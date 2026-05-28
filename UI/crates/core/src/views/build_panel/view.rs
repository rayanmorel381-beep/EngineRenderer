use crate::editor::build::{BuildConfig, BuildStatus, BuildTarget};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown};

pub struct BuildPanelView {
    target_sel: usize,
}

impl Default for BuildPanelView {
    fn default() -> Self { Self { target_sel: 0 } }
}

impl BuildPanelView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, config: &mut BuildConfig, status: &BuildStatus, on_build: &mut bool) {
        let panel = Panel::new("Build & Déploiement").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("build_panel");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let target_opts: Vec<&str> = BuildTarget::ALL.iter().map(|t| t.label()).collect();
        self.target_sel = BuildTarget::ALL.iter().position(|t| *t == config.target).unwrap_or(0);
        if Dropdown::new("Cible", &target_opts).show(ui, id.child("tgt"), Rect::new(lx, y, w, row_h), &mut self.target_sel) {
            config.target = BuildTarget::ALL[self.target_sel].clone();
        }
        y += row_h + sp;

        let rel_lbl = if config.release_mode { "Release: ON" } else { "Release: OFF" };
        if Button::new(rel_lbl).with_style(if config.release_mode { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("rel"), Rect::new(lx, y, (w * 0.48).floor(), row_h)).clicked { config.release_mode = !config.release_mode; }
        let ba_lbl = if config.bundle_assets { "Assets: Bundlés" } else { "Assets: Séparés" };
        if Button::new(ba_lbl).with_style(if config.bundle_assets { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("ba"), Rect::new(lx + (w * 0.52).floor(), y, (w * 0.48).floor(), row_h)).clicked { config.bundle_assets = !config.bundle_assets; }
        y += row_h + sp;

        let strip_lbl = if config.strip_debug { "Strip Debug: ON" } else { "Strip Debug: OFF" };
        if Button::new(strip_lbl).with_style(if config.strip_debug { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("strip"), Rect::new(lx, y, w, row_h)).clicked { config.strip_debug = !config.strip_debug; }
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Sortie: {}", config.output_path), p.text_muted, m.font_size_normal);
        y += m.font_size_normal + sp;

        let can_build = !matches!(status, BuildStatus::Building(_));
        if Button::primary("Construire").enabled(can_build)
            .show(ui, id.child("build"), Rect::new(lx, y, w, row_h + 4.0)).clicked { *on_build = true; }
        y += row_h + 4.0 + sp;

        let status_color = match status {
            BuildStatus::Success => p.success,
            BuildStatus::Failed(_) => p.error,
            BuildStatus::Building(_) => p.warning,
            BuildStatus::Idle => p.text_muted,
        };
        ui.draw_list.text(Vec2::new(lx, y), &status.label(), status_color, m.font_size_normal);

        if let BuildStatus::Building(progress) = status {
            y += m.font_size_normal + 4.0;
            let bar_full = Rect::new(lx, y, w, 8.0);
            ui.draw_list.rect(bar_full, p.panel_active, 4.0);
            ui.draw_list.rect(Rect::new(lx, y, w * progress.clamp(0.0, 1.0), 8.0), p.accent, 4.0);
        }
    }
}

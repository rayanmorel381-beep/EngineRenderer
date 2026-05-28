use crate::scene::network::{NetworkRole, NetworkState};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown, Slider};

pub struct NetworkEditorView {
    pub role_idx: usize,
}

impl Default for NetworkEditorView {
    fn default() -> Self { Self { role_idx: 0 } }
}

impl NetworkEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, state: &mut NetworkState) {
        let panel = Panel::new("Réseau & Réplication").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("net_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let role_labels: Vec<&str> = NetworkRole::ALL.iter().map(|r| r.label()).collect();
        if Dropdown::new("Rôle réseau", &role_labels).show(ui, id.child("role"), Rect::new(lx, y, w, row_h), &mut self.role_idx) {
            state.role = NetworkRole::ALL[self.role_idx].clone();
        }
        y += row_h + sp;

        let ded_lbl = if state.dedicated { "Dédié: Oui" } else { "Dédié: Non" };
        if Button::new(ded_lbl).with_style(if state.dedicated { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("ded"), Rect::new(lx, y, 140.0, row_h)).clicked { state.dedicated = !state.dedicated; }
        y += row_h + sp;

        let mut max_f = state.max_clients as f64;
        if Slider::new("Max clients", 2.0, 128.0).show(ui, id.child("maxc"), Rect::new(lx, y, w, row_h), &mut max_f) {
            state.max_clients = max_f as usize;
        }
        y += row_h + sp;
        Slider::new("FPS serveur", 10.0, 128.0).show(ui, id.child("sfps"), Rect::new(lx, y, w, row_h), &mut state.server_fps);
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Ping: {:.1} ms", state.ping_ms), p.text_muted, m.font_size_normal);
        y += m.font_size_normal + sp;
        ui.draw_list.text(Vec2::new(lx, y), &format!("Perte paquets: {:.1}%", state.packet_loss * 100.0), p.text_muted, m.font_size_normal);
        y += m.font_size_normal + sp;
        ui.draw_list.text(Vec2::new(lx, y), &format!("Tick réseau: {}", state.tick), p.text_muted, m.font_size_normal);
    }
}

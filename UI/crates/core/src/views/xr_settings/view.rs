use crate::scene::xr::{XrMode, XrSettings, XrTrackingOrigin};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown, Slider};

pub struct XrSettingsView {
    pub mode_idx: usize,
    pub origin_idx: usize,
}

impl Default for XrSettingsView {
    fn default() -> Self { Self { mode_idx: 0, origin_idx: 0 } }
}

impl XrSettingsView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, xr: &mut XrSettings) {
        let panel = Panel::new("Paramètres XR / VR").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("xr_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let mode_labels: Vec<&str> = XrMode::ALL.iter().map(|mo| mo.label()).collect();
        if Dropdown::new("Mode XR", &mode_labels).show(ui, id.child("mode"), Rect::new(lx, y, w, row_h), &mut self.mode_idx) {
            xr.mode = XrMode::ALL[self.mode_idx].clone();
        }
        y += row_h + sp;

        let origin_labels: Vec<&str> = XrTrackingOrigin::ALL.iter().map(|o| o.label()).collect();
        if Dropdown::new("Origine tracking", &origin_labels).show(ui, id.child("origin"), Rect::new(lx, y, w, row_h), &mut self.origin_idx) {
            xr.tracking_origin = XrTrackingOrigin::ALL[self.origin_idx].clone();
        }
        y += row_h + sp;

        Slider::new("Échelle rendu", 0.5, 2.0).show(ui, id.child("rscale"), Rect::new(lx, y, w, row_h), &mut xr.render_scale);
        y += row_h + sp;
        Slider::new("IPD (mm)", 52.0, 78.0).show(ui, id.child("ipd"), Rect::new(lx, y, w, row_h), &mut xr.ipd_mm);
        y += row_h + sp;
        Slider::new("Taux rafr. (Hz)", 60.0, 144.0).show(ui, id.child("rr"), Rect::new(lx, y, w, row_h), &mut xr.refresh_rate);
        y += row_h + sp;
        Slider::new("FOV horizontal °", 60.0, 130.0).show(ui, id.child("fovh"), Rect::new(lx, y, w, row_h), &mut xr.fov_h_deg);
        y += row_h + sp;
        Slider::new("FOV vertical °", 60.0, 130.0).show(ui, id.child("fovv"), Rect::new(lx, y, w, row_h), &mut xr.fov_v_deg);
        y += row_h + sp;

        let mut fov_f = xr.foveation_level as f64;
        if Slider::new("Fovéation", 0.0, 4.0).show(ui, id.child("fov"), Rect::new(lx, y, w, row_h), &mut fov_f) {
            xr.foveation_level = fov_f as u8;
        }
        y += row_h + sp;

        let ht_lbl = if xr.hand_tracking { "Tracking mains: ON" } else { "Tracking mains: OFF" };
        if Button::new(ht_lbl).with_style(if xr.hand_tracking { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("ht"), Rect::new(lx, y, 200.0, row_h)).clicked { xr.hand_tracking = !xr.hand_tracking; }
        y += row_h + sp;

        let et_lbl = if xr.eye_tracking { "Eye tracking: ON" } else { "Eye tracking: OFF" };
        if Button::new(et_lbl).with_style(if xr.eye_tracking { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("et"), Rect::new(lx, y, 200.0, row_h)).clicked { xr.eye_tracking = !xr.eye_tracking; }
        y += row_h + sp;

        let pt_lbl = if xr.passthrough { "Passthrough: ON" } else { "Passthrough: OFF" };
        if Button::new(pt_lbl).with_style(if xr.passthrough { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("pt"), Rect::new(lx, y, 180.0, row_h)).clicked { xr.passthrough = !xr.passthrough; }
        y += row_h + sp;

        if xr.hand_tracking {
            ui.draw_list.text(Vec2::new(lx, y), &format!("Main G: ({:.2}, {:.2}, {:.2})", xr.left_hand.position[0], xr.left_hand.position[1], xr.left_hand.position[2]), p.text_muted, m.font_size_small);
            y += m.font_size_small + 2.0;
            ui.draw_list.text(Vec2::new(lx, y), &format!("Main D: ({:.2}, {:.2}, {:.2})", xr.right_hand.position[0], xr.right_hand.position[1], xr.right_hand.position[2]), p.text_muted, m.font_size_small);
        }
    }
}

use crate::scene::input_map::{InputAction, InputAxis, InputBinding, InputDevice, InputMap, InputTrigger};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct InputMapperView {
    pub active_tab: usize,
    pub selected_action: Option<usize>,
    pub selected_axis: Option<usize>,
    pub editing_binding: Option<(usize, usize)>,
}

impl Default for InputMapperView {
    fn default() -> Self { Self { active_tab: 0, selected_action: None, selected_axis: None, editing_binding: None } }
}

impl InputMapperView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, map: &mut InputMap) {
        let panel = Panel::new("Mappings d'entrée").with_icon(Icon::Input);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("input_map");
        let (tab_area, rest) = body.split_top(row_h + sp);
        let tab_w = (tab_area.width - pad * 2.0) / 2.0;
        let tabs = ["Actions", "Axes"];
        for (ti, tab_lbl) in tabs.iter().enumerate() {
            let tr = Rect::new(tab_area.x + pad + ti as f64 * tab_w, tab_area.y + sp * 0.5, tab_w - 4.0, row_h);
            let style = if self.active_tab == ti { ButtonStyle::Primary } else { ButtonStyle::Secondary };
            if Button::new(tab_lbl).with_style(style).show(ui, id.child(&format!("tab{ti}")), tr).clicked {
                self.active_tab = ti;
            }
        }
        let mut y = rest.y + sp;
        let lx = rest.x + pad;
        let w = rest.width - pad * 2.0;

        if self.active_tab == 0 {
            let mut remove_action: Option<usize> = None;
            for (i, action) in map.actions.iter_mut().enumerate() {
                let sel = self.selected_action == Some(i);
                let row = Rect::new(lx, y, w, row_h);
                ui.draw_list.rect(row, if sel { p.panel_active } else { p.panel }, 2.0);
                ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &action.name, p.text, m.font_size_small);
                let bind_str = action.bindings.iter().map(|b| b.key.as_str()).collect::<Vec<_>>().join(" / ");
                ui.draw_list.text(Vec2::new(lx + 120.0, y + (row_h - m.font_size_small) * 0.5), &bind_str, p.text_muted, m.font_size_small);
                let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
                let del_r = Rect::new(lx + w - 34.0, y + 2.0, 32.0, row_h - 4.0);
                if Button::new("▼").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("asel{i}")), sel_r).clicked {
                    self.selected_action = if sel { None } else { Some(i) };
                }
                if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("adel{i}")), del_r).clicked {
                    remove_action = Some(i);
                }
                y += row_h + 2.0;
                if sel {
                    let en_lbl = if action.enabled { "Actif" } else { "Inactif" };
                    if Button::new(en_lbl).with_style(if action.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                        .show(ui, id.child(&format!("aen{i}")), Rect::new(lx + pad, y, 80.0, row_h)).clicked {
                        action.enabled = !action.enabled;
                    }
                    y += row_h + sp;
                    for (bi, bind) in action.bindings.iter().enumerate() {
                        let brow = Rect::new(lx + pad, y, w - pad, m.font_size_small + 4.0);
                        ui.draw_list.rect(brow, p.panel, 1.0);
                        ui.draw_list.text(Vec2::new(brow.x + 4.0, brow.y), &format!("[{}] {} — {} — {}", bi + 1, bind.device.label(), bind.key, bind.trigger.label()), p.text_muted, m.font_size_small);
                        y += m.font_size_small + 6.0;
                    }
                    if Button::new("+ Liaison").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("abind{i}")), Rect::new(lx + pad, y, w - pad, row_h)).clicked {
                        action.bindings.push(InputBinding::new("", InputDevice::Keyboard, InputTrigger::Pressed));
                    }
                    y += row_h + sp;
                }
            }
            if let Some(idx) = remove_action { map.remove_action(idx); if self.selected_action == Some(idx) { self.selected_action = None; } }
            if Button::new("+ Action").with_style(ButtonStyle::Secondary).show(ui, id.child("add_action"), Rect::new(lx, y, w, row_h)).clicked {
                map.add_action(InputAction::new(format!("Action {}", map.actions.len() + 1)));
            }
        } else {
            let mut remove_axis: Option<usize> = None;
            for (i, axis) in map.axes.iter_mut().enumerate() {
                let sel = self.selected_axis == Some(i);
                let row = Rect::new(lx, y, w, row_h);
                ui.draw_list.rect(row, if sel { p.panel_active } else { p.panel }, 2.0);
                ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &axis.name, p.text, m.font_size_small);
                let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
                let del_r = Rect::new(lx + w - 34.0, y + 2.0, 32.0, row_h - 4.0);
                if Button::new("▼").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("xsel{i}")), sel_r).clicked {
                    self.selected_axis = if sel { None } else { Some(i) };
                }
                if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("xdel{i}")), del_r).clicked {
                    remove_axis = Some(i);
                }
                y += row_h + 2.0;
                if sel {
                    Slider::new("Zone morte", 0.0, 0.5).show(ui, id.child(&format!("xdz{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut axis.dead_zone);
                    y += row_h + sp;
                    Slider::new("Sensibilité", 0.1, 5.0).show(ui, id.child(&format!("xse{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut axis.sensitivity);
                    y += row_h + sp;
                }
            }
            if let Some(idx) = remove_axis { map.remove_axis(idx); if self.selected_axis == Some(idx) { self.selected_axis = None; } }
            if Button::new("+ Axe").with_style(ButtonStyle::Secondary).show(ui, id.child("add_axis"), Rect::new(lx, y, w, row_h)).clicked {
                map.add_axis(InputAxis::new(format!("Axe {}", map.axes.len() + 1)));
            }
        }
    }
}

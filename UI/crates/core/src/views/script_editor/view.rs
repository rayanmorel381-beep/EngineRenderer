use crate::scene::script::ScriptComponent;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle};

pub struct ScriptEditorView {
    pub selected_script: usize,
    scroll_y: f64,
    pub pending_add: bool,
    pub pending_remove: Option<usize>,
    pub pending_toggle: Option<usize>,
}

impl Default for ScriptEditorView {
    fn default() -> Self {
        Self {
            selected_script: 0,
            scroll_y: 0.0,
            pending_add: false,
            pending_remove: None,
            pending_toggle: None,
        }
    }
}

impl ScriptEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, scripts: &[ScriptComponent]) {
        let panel = Panel::new("Scripts").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;

        let (list_rect, editor_rect) = body.split_left((body.width * 0.30).clamp(100.0, 200.0));

        ui.draw_list.rect(list_rect, p.panel, 0.0);

        let add_r = Rect::new(list_rect.x + pad, list_rect.y + pad, list_rect.width - pad * 2.0, row_h);
        if Button::icon(Icon::Add).show(ui, WidgetId::hash_str("sc_add"), add_r).clicked {
            self.pending_add = true;
        }

        let list_body = Rect::new(
            list_rect.x,
            list_rect.y + pad + row_h + 4.0,
            list_rect.width,
            list_rect.height - pad - row_h - 4.0,
        );
        ui.draw_list.push_clip(list_body);
        for (i, script) in scripts.iter().enumerate() {
            let y = list_body.y + i as f64 * (row_h + 2.0) - self.scroll_y;
            if y + row_h < list_body.y || y > list_body.y + list_body.height { continue; }
            let active = self.selected_script == i;
            let bg = if active { p.accent } else { p.background };
            ui.draw_list.rect(Rect::new(list_body.x, y, list_body.width, row_h), bg, 2.0);
            let tc = if script.enabled { p.text } else { p.text_muted };
            ui.draw_list.text(Vec2::new(list_body.x + pad, y + (row_h - m.font_size_normal) * 0.5), &script.name, tc, m.font_size_normal);
            let sel_id = WidgetId::hash_str("sc_sel_").combine(WidgetId(i as u64));
            if Button::new("").show(ui, sel_id, Rect::new(list_body.x, y, list_body.width, row_h)).clicked {
                self.selected_script = i;
            }
        }
        ui.draw_list.pop_clip();

        let scroll_delta = ui.input.pointer.scroll_y;
        if ui.input.pointer.x >= list_rect.x && ui.input.pointer.x <= list_rect.x + list_rect.width {
            let max_scroll = (scripts.len() as f64 * (row_h + 2.0) - list_body.height).max(0.0);
            self.scroll_y = (self.scroll_y - scroll_delta * 12.0).clamp(0.0, max_scroll);
        }

        let er = Rect::new(editor_rect.x + pad, editor_rect.y, editor_rect.width - pad, editor_rect.height);

        if scripts.is_empty() {
            ui.draw_list.text(
                Vec2::new(er.x + pad, er.y + pad),
                "Aucun script — cliquez + pour en ajouter un",
                p.text_muted, m.font_size_normal,
            );
            return;
        }

        let Some(script) = scripts.get(self.selected_script) else { return };

        let btn_h = row_h;
        let btn_w = 90.0;
        let mut bx = er.x;

        let toggle_style = if script.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary };
        let toggle_label = if script.enabled { "Activé" } else { "Désactivé" };
        if Button::new(toggle_label).with_style(toggle_style).show(
            ui, WidgetId::hash_str("sc_toggle"),
            Rect::new(bx, er.y + pad, btn_w, btn_h),
        ).clicked {
            self.pending_toggle = Some(self.selected_script);
        }
        bx += btn_w + 4.0;

        if Button::new("Supprimer").with_style(ButtonStyle::Secondary).show(
            ui, WidgetId::hash_str("sc_del"),
            Rect::new(bx, er.y + pad, btn_w, btn_h),
        ).clicked {
            self.pending_remove = Some(self.selected_script);
        }

        let code_y = er.y + pad + btn_h + 8.0;
        let code_rect = Rect::new(er.x, code_y, er.width, (er.y + er.height - code_y).max(0.0));
        ui.draw_list.rect(code_rect, p.background, 2.0);
        ui.draw_list.rect_outline(code_rect, p.panel_active, 1.0, 2.0);

        let source = if script.source.is_empty() { script.on_update_stub() } else { script.source.clone() };
        let line_h = m.font_size_normal * 1.4;
        ui.draw_list.push_clip(code_rect);
        for (i, line) in source.lines().enumerate() {
            let ly = code_rect.y + pad + i as f64 * line_h;
            if ly > code_rect.y + code_rect.height { break; }
            let ln = format!("{:>3}  ", i + 1);
            ui.draw_list.text(Vec2::new(code_rect.x + 2.0, ly), &ln, p.text_muted, m.font_size_normal);
            ui.draw_list.text(Vec2::new(code_rect.x + 30.0, ly), line, p.text, m.font_size_normal);
        }
        ui.draw_list.pop_clip();
    }
}

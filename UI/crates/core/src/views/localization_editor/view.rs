use crate::scene::localization::LocalizationTable;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle};

pub struct LocalizationEditorView {
    pub selected_locale_idx: usize,
    pub selected_entry: Option<usize>,
}

impl Default for LocalizationEditorView {
    fn default() -> Self { Self { selected_locale_idx: 0, selected_entry: None } }
}

impl LocalizationEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, table: &mut LocalizationTable) {
        let panel = Panel::new("Localisation i18n").with_icon(Icon::Globe);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("loc_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), "Langue active:", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let locale_w = (w - sp * (table.locales.len() as f64 - 1.0)) / table.locales.len().max(1) as f64;
        for (li, locale) in table.locales.iter().enumerate() {
            let is_active = *locale == table.active_locale;
            let lr = Rect::new(lx + li as f64 * (locale_w + sp), y, locale_w, row_h);
            let style = if is_active { ButtonStyle::Primary } else { ButtonStyle::Secondary };
            if Button::new(locale.as_str()).with_style(style).show(ui, id.child(&format!("lang{li}")), lr).clicked {
                table.active_locale = locale.clone();
                self.selected_locale_idx = li;
            }
        }
        y += row_h + sp;

        let missing = table.missing_translations();
        if !missing.is_empty() {
            let miss_lbl = format!("{} traductions manquantes", missing.len());
            ui.draw_list.text(Vec2::new(lx, y), &miss_lbl, p.warning, m.font_size_small);
            y += m.font_size_small + sp;
        }

        ui.draw_list.text(Vec2::new(lx, y), "Clé", p.text_muted, m.font_size_small);
        let trans_col_x = lx + w * 0.45;
        ui.draw_list.text(Vec2::new(trans_col_x, y), &table.active_locale, p.text_muted, m.font_size_small);
        y += m.font_size_small + 4.0;
        ui.draw_list.line(Vec2::new(lx, y), Vec2::new(lx + w, y), p.border, 1.0);
        y += 3.0;

        let mut remove_entry: Option<usize> = None;
        for (ei, entry) in table.entries.iter().enumerate() {
            let sel = self.selected_entry == Some(ei);
            let er = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(er, if sel { p.panel_active } else { if ei % 2 == 0 { p.panel } else { p.panel_hover } }, 1.0);
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &entry.key, p.text, m.font_size_small);
            let trans = entry.get(&table.active_locale).unwrap_or("[manquant]");
            let trans_col = if trans == "[manquant]" { p.warning } else { p.text_muted };
            ui.draw_list.text(Vec2::new(trans_col_x, y + (row_h - m.font_size_small) * 0.5), trans, trans_col, m.font_size_small);
            let del_r = Rect::new(lx + w - 20.0, y + 2.0, 18.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("edel{ei}")), del_r).clicked {
                remove_entry = Some(ei);
            }
            if ui.is_rect_hovered(er) { self.selected_entry = Some(ei); }
            y += row_h + 2.0;
        }
        if let Some(idx) = remove_entry {
            table.remove_entry(idx);
            if self.selected_entry == Some(idx) { self.selected_entry = None; }
        }
        if Button::new("+ Clé").with_style(ButtonStyle::Secondary).show(ui, id.child("add_entry"), Rect::new(lx, y, 80.0, row_h)).clicked {
            table.add_entry(format!("ui.key_{}", table.entries.len()));
        }
    }
}

use crate::scene::data_table::{DataFieldType, DataTable, DataTableLibrary};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle};

pub struct DataTableEditorView {
    pub selected_table: Option<usize>,
    pub selected_row: Option<usize>,
}

impl Default for DataTableEditorView {
    fn default() -> Self { Self { selected_table: None, selected_row: None } }
}

impl DataTableEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, lib: &mut DataTableLibrary) {
        let panel = Panel::new("Tables de données").with_icon(Icon::Database);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("dtable_ed");

        let (table_list_r, data_r) = body.split_left(160.0);
        let tlx = table_list_r.x + pad;
        let tw = table_list_r.width - pad * 2.0;
        let mut ty = table_list_r.y + pad;

        ui.draw_list.text(Vec2::new(tlx, ty), "Tables", p.text, m.font_size_normal);
        ty += m.font_size_normal + sp;

        let mut remove_table: Option<usize> = None;
        for (i, table) in lib.tables.iter().enumerate() {
            let sel = self.selected_table == Some(i);
            let tr = Rect::new(tlx, ty, tw, row_h);
            ui.draw_list.rect(tr, if sel { p.panel_active } else { p.panel }, 2.0);
            ui.draw_list.text(Vec2::new(tlx + 4.0, ty + (row_h - m.font_size_small) * 0.5), &table.name, p.text, m.font_size_small);
            let del_r = Rect::new(tlx + tw - 20.0, ty + 2.0, 18.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("tdel{i}")), del_r).clicked {
                remove_table = Some(i);
            }
            if ui.is_rect_hovered(tr) { self.selected_table = Some(i); }
            ty += row_h + 2.0;
        }
        if let Some(idx) = remove_table {
            lib.tables.remove(idx);
            if self.selected_table == Some(idx) { self.selected_table = None; self.selected_row = None; }
        }
        if Button::new("+ Table").with_style(ButtonStyle::Secondary).show(ui, id.child("add_t"), Rect::new(tlx, ty, tw, row_h)).clicked {
            let t = DataTable::new(format!("Table {}", lib.tables.len() + 1));
            lib.tables.push(t);
        }

        if let Some(tidx) = self.selected_table {
            if tidx < lib.tables.len() {
                let table = &mut lib.tables[tidx];
                let dx = data_r.x + pad;
                let dw = data_r.width - pad * 2.0;
                let mut dy = data_r.y + pad;

                let header_lbl = format!("{} — {} colonnes, {} lignes", table.name, table.columns.len(), table.rows.len());
                ui.draw_list.text(Vec2::new(dx, dy), &header_lbl, p.text, m.font_size_normal);
                dy += m.font_size_normal + sp;

                let col_w = if table.columns.is_empty() { 80.0 } else { (dw - 80.0) / table.columns.len() as f64 };
                ui.draw_list.text(Vec2::new(dx, dy), "Ligne", p.text_muted, m.font_size_small);
                for (ci, col) in table.columns.iter().enumerate() {
                    ui.draw_list.text(Vec2::new(dx + 80.0 + ci as f64 * col_w, dy), &col.name, p.text_muted, m.font_size_small);
                }
                dy += m.font_size_small + 4.0;
                ui.draw_list.line(Vec2::new(dx, dy), Vec2::new(dx + dw, dy), p.border, 1.0);
                dy += 3.0;

                let mut remove_row: Option<usize> = None;
                for (ri, row) in table.rows.iter().enumerate() {
                    let sel = self.selected_row == Some(ri);
                    let rr = Rect::new(dx, dy, dw, row_h);
                    ui.draw_list.rect(rr, if sel { p.panel_active } else { p.panel_hover }, 1.0);
                    ui.draw_list.text(Vec2::new(dx + 4.0, dy + (row_h - m.font_size_small) * 0.5), &row.name, p.text, m.font_size_small);
                    for (ci, cell) in row.cells.iter().enumerate() {
                        ui.draw_list.text(Vec2::new(dx + 80.0 + ci as f64 * col_w, dy + (row_h - m.font_size_small) * 0.5), &cell.label(), p.text_muted, m.font_size_small);
                    }
                    let del_r = Rect::new(dx + dw - 20.0, dy + 2.0, 18.0, row_h - 4.0);
                    if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("rdel{ri}")), del_r).clicked {
                        remove_row = Some(ri);
                    }
                    if ui.is_rect_hovered(rr) { self.selected_row = Some(ri); }
                    dy += row_h + 2.0;
                }
                if let Some(ridx) = remove_row {
                    table.remove_row(ridx);
                    if self.selected_row == Some(ridx) { self.selected_row = None; }
                }
                if Button::new("+ Ligne").with_style(ButtonStyle::Secondary).show(ui, id.child("add_row"), Rect::new(dx, dy, 80.0, row_h)).clicked {
                    let new_id = table.add_row(format!("Ligne {}", table.rows.len() + 1));
                    self.selected_row = table.rows.iter().position(|r| r.id == new_id);
                }
                if Button::new("+ Colonne").with_style(ButtonStyle::Secondary).show(ui, id.child("add_col"), Rect::new(dx + 86.0, dy, 100.0, row_h)).clicked {
                    table.add_column(format!("Col {}", table.columns.len() + 1), DataFieldType::String);
                }
            }
        }
    }
}

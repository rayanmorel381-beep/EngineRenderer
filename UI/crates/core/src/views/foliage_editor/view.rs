use crate::scene::foliage::{FoliagePaintMode, FoliagePainter};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct FoliageEditorView {
    pub selected_type: Option<usize>,
    pub scatter_seed: u64,
}

impl Default for FoliageEditorView {
    fn default() -> Self { Self { selected_type: None, scatter_seed: 42 } }
}

impl FoliageEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, painter: &mut FoliagePainter) {
        let panel = Panel::new("Foliage Painter").with_icon(Icon::Tree);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("foliage_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let stats = format!("Total instances: {}  |  Types: {}", painter.total_instances(), painter.foliage_types.len());
        ui.draw_list.text(Vec2::new(lx, y), &stats, p.text_muted, m.font_size_small);
        y += m.font_size_small + sp;

        Slider::new("Rayon brosse", 0.5, 50.0).show(ui, id.child("brad"), Rect::new(lx, y, w, row_h), &mut painter.brush_radius);
        y += row_h + sp;
        Slider::new("Densité", 0.1, 10.0).show(ui, id.child("bdens"), Rect::new(lx, y, w, row_h), &mut painter.brush_density);
        y += row_h + sp;
        Slider::new("Force", 0.0, 1.0).show(ui, id.child("bstr"), Rect::new(lx, y, w, row_h), &mut painter.brush_strength);
        y += row_h + sp;

        let mode_lbl = painter.mode.label();
        if Button::new(mode_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child("mode"), Rect::new(lx, y, 120.0, row_h)).clicked {
            painter.mode = match painter.mode {
                FoliagePaintMode::Add => FoliagePaintMode::Remove,
                FoliagePaintMode::Remove => FoliagePaintMode::Paint,
                FoliagePaintMode::Paint => FoliagePaintMode::Add,
            };
        }
        y += row_h + sp;

        ui.draw_list.text(Vec2::new(lx, y), "Types de foliage", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let mut remove_type: Option<usize> = None;
        for (i, ftype) in painter.foliage_types.iter_mut().enumerate() {
            let sel = self.selected_type == Some(i);
            let row = Rect::new(lx, y, w, row_h);
            ui.draw_list.rect(row, if sel { p.panel_active } else { p.panel }, 2.0);
            let info = format!("{} — {} instances", ftype.name, ftype.instances.len());
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_small) * 0.5), &info, p.text, m.font_size_small);
            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 34.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("▼").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("tsel{i}")), sel_r).clicked {
                self.selected_type = if sel { None } else { Some(i) };
                painter.active_type = i;
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("tdel{i}")), del_r).clicked {
                remove_type = Some(i);
            }
            y += row_h + 2.0;
            if sel {
                Slider::new("Densité", 0.0, 5.0).show(ui, id.child(&format!("td{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut ftype.density);
                y += row_h + sp;
                Slider::new("Échelle min", 0.1, 3.0).show(ui, id.child(&format!("tsmin{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut ftype.min_scale);
                y += row_h + sp;
                Slider::new("Échelle max", 0.1, 3.0).show(ui, id.child(&format!("tsmax{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut ftype.max_scale);
                y += row_h + sp;
                Slider::new("Dist. culling", 10.0, 1000.0).show(ui, id.child(&format!("tcull{i}")), Rect::new(lx + pad, y, w - pad, row_h), &mut ftype.cull_distance);
                y += row_h + sp;
                let scatter_r = Rect::new(lx + pad, y, (w - pad - sp) * 0.5, row_h);
                let clear_r = Rect::new(lx + pad + (w - pad - sp) * 0.5 + sp, y, (w - pad - sp) * 0.5, row_h);
                if Button::new("Scatter").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("scatter{i}")), scatter_r).clicked {
                    ftype.scatter([0.0, 0.0, 0.0], painter.brush_radius, (painter.brush_density * 20.0) as usize, self.scatter_seed);
                    self.scatter_seed = self.scatter_seed.wrapping_add(1);
                }
                if Button::new("Vider").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("clear{i}")), clear_r).clicked {
                    ftype.instances.clear();
                }
                y += row_h + sp;
            }
        }
        if let Some(idx) = remove_type {
            painter.foliage_types.remove(idx);
            if self.selected_type == Some(idx) { self.selected_type = None; }
        }
        if Button::new("+ Type").with_style(ButtonStyle::Secondary).show(ui, id.child("add_type"), Rect::new(lx, y, w, row_h)).clicked {
            painter.foliage_types.push(crate::scene::foliage::FoliageType::new(format!("Type {}", painter.foliage_types.len() + 1)));
        }
    }
}

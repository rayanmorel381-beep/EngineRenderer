use crate::scene::terrain::TerrainData;
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

#[derive(Clone, Debug, PartialEq)]
pub enum TerrainTool {
    Raise,
    Lower,
    Smooth,
    Paint,
}

impl TerrainTool {
    pub fn label(&self) -> &'static str {
        match self { Self::Raise => "Élever", Self::Lower => "Abaisser", Self::Smooth => "Lisser", Self::Paint => "Peindre" }
    }
}

pub struct TerrainEditorView {
    pub active_tool: TerrainTool,
    pub brush_radius: f64,
    pub brush_strength: f64,
    pub selected_layer: usize,
    pub terrain: Option<TerrainData>,
}

impl Default for TerrainEditorView {
    fn default() -> Self {
        Self {
            active_tool: TerrainTool::Raise,
            brush_radius: 5.0,
            brush_strength: 0.5,
            selected_layer: 0,
            terrain: None,
        }
    }
}

impl TerrainEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Terrain Editor").with_icon(Icon::Grid);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("terr_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), "Outil", p.text_muted, m.font_size_normal);
        y += m.font_size_normal + 4.0;
        let tools = [TerrainTool::Raise, TerrainTool::Lower, TerrainTool::Smooth, TerrainTool::Paint];
        let btn_w = (w - sp * 3.0) / 4.0;
        for (i, tool) in tools.iter().enumerate() {
            let active = self.active_tool == *tool;
            let r = Rect::new(lx + i as f64 * (btn_w + sp), y, btn_w, row_h);
            if Button::new(tool.label())
                .with_style(if active { ButtonStyle::Primary } else { ButtonStyle::Secondary })
                .show(ui, id.child(tool.label()), r).clicked { self.active_tool = tool.clone(); }
        }
        y += row_h + sp;

        Slider::new("Rayon", 0.5, 50.0).show(ui, id.child("rad"), Rect::new(lx, y, w, row_h), &mut self.brush_radius);
        y += row_h + sp;
        Slider::new("Force", 0.01, 1.0).show(ui, id.child("str"), Rect::new(lx, y, w, row_h), &mut self.brush_strength);
        y += row_h + sp;

        if let Some(terrain) = &self.terrain {
            if self.active_tool == TerrainTool::Paint {
                ui.draw_list.text(Vec2::new(lx, y), "Couche", p.text_muted, m.font_size_normal);
                y += m.font_size_normal + 4.0;
                for (i, layer) in terrain.layers.iter().enumerate() {
                    let active = self.selected_layer == i;
                    let r = Rect::new(lx, y, w, row_h);
                    let bg = if active { p.accent } else { p.panel_active };
                    ui.draw_list.rect(r, bg, 3.0);
                    let swatch = Rect::new(lx, y, 12.0, row_h);
                    ui.draw_list.rect(swatch, layer.color, 2.0);
                    ui.draw_list.text(Vec2::new(lx + 16.0, y + (row_h - m.font_size_normal) * 0.5), &layer.name, p.text, m.font_size_normal);
                    if Button::new("").show(ui, id.child(&format!("lay{i}")), r).clicked {
                        self.selected_layer = i;
                    }
                    y += row_h + 2.0;
                }
            }

            y += sp;
            ui.draw_list.text(Vec2::new(lx, y), &format!("{}×{} terrain  rés={:.1}", terrain.width, terrain.height, terrain.resolution), p.text_muted, m.font_size_normal);
        } else {
            if Button::primary("Créer terrain 64×64").show(ui, id.child("create"), Rect::new(lx, y, w, row_h)).clicked {
                self.terrain = Some(TerrainData::new(64, 64, 1.0));
            }
        }
    }
}

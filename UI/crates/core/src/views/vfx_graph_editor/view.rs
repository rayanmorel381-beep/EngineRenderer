use crate::scene::vfx_graph::{VfxGraph, VfxNode, VfxNodeCategory};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct VfxGraphEditorView {
    pub scroll_offset: [f64; 2],
    pub zoom: f64,
    pub selected_node: Option<u32>,
    pub category_filter: usize,
}

impl Default for VfxGraphEditorView {
    fn default() -> Self { Self { scroll_offset: [0.0; 2], zoom: 1.0, selected_node: None, category_filter: 0 } }
}

impl VfxGraphEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, graph: &mut VfxGraph) {
        let title = format!("VFX Graph — {}", graph.name);
        let panel = Panel::new(&title).with_icon(Icon::Particles);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("vfx_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let (top_bar, canvas_area) = body.split_top(row_h + sp * 2.0);
        let lx_top = top_bar.x + pad;
        let mut tx = lx_top;
        let en_lbl = if graph.enabled { "Activé" } else { "Désactivé" };
        if Button::new(en_lbl).with_style(if graph.enabled { ButtonStyle::Primary } else { ButtonStyle::Secondary })
            .show(ui, id.child("en"), Rect::new(tx, y, 80.0, row_h)).clicked {
            graph.enabled = !graph.enabled;
        }
        tx += 86.0;
        let loop_lbl = if graph.looping { "Boucle: ON" } else { "Boucle: OFF" };
        if Button::new(loop_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child("loop"), Rect::new(tx, y, 100.0, row_h)).clicked {
            graph.looping = !graph.looping;
        }
        tx += 106.0;
        let stats = format!("Max: {} particules | Durée: {:.1}s", graph.max_particles, graph.loop_duration);
        ui.draw_list.text(Vec2::new(tx, y + (row_h - m.font_size_small) * 0.5), &stats, p.text_muted, m.font_size_small);
        y = canvas_area.y + sp;

        let node_w = 160.0;
        let node_h = 80.0;
        ui.draw_list.rect(canvas_area, p.viewport_clear, 0.0);

        let mut remove_node: Option<u32> = None;
        for node in &mut graph.nodes {
            let nx = canvas_area.x + node.position[0] * self.zoom + self.scroll_offset[0];
            let ny = canvas_area.y + node.position[1] * self.zoom + self.scroll_offset[1];
            if nx + node_w < canvas_area.x || nx > canvas_area.x + canvas_area.width { continue; }
            if ny + node_h < canvas_area.y || ny > canvas_area.y + canvas_area.height { continue; }
            let node_rect = Rect::new(nx, ny, node_w, node_h);
            let border_col = if self.selected_node == Some(node.id) { p.accent } else { p.border };
            ui.draw_list.rect(node_rect, p.panel, 4.0);
            ui.draw_list.rect_outline(node_rect, border_col, 4.0, 1.5);
            let header_h = 20.0;
            let header_rect = Rect::new(nx, ny, node_w, header_h);
            let cat_col = cat_color(&node.category, p.accent);
            ui.draw_list.rect(header_rect, cat_col, 4.0);
            ui.draw_list.text(Vec2::new(nx + 4.0, ny + 3.0), &node.title, p.text, m.font_size_small);
            let del_r = Rect::new(nx + node_w - 18.0, ny + 1.0, 16.0, 16.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("ndel{}", node.id)), del_r).clicked {
                remove_node = Some(node.id);
            }
            if ui.is_rect_hovered(node_rect) { self.selected_node = Some(node.id); }
            let val_str = format!("{:.2}", node.value_float);
            ui.draw_list.text(Vec2::new(nx + 6.0, ny + header_h + 6.0), &val_str, p.text_muted, m.font_size_small);
        }
        if let Some(nid) = remove_node { graph.remove_node(nid); self.selected_node = None; }
        let mut dur_f = graph.loop_duration;
        Slider::new("Durée (s)", 0.5, 30.0).show(ui, id.child("dur"), Rect::new(lx, y, 200.0, row_h), &mut dur_f);
        graph.loop_duration = dur_f;
        y += row_h + sp;
        let mut max_f = graph.max_particles as f64;
        Slider::new("Max particules", 10.0, 10000.0).show(ui, id.child("maxp"), Rect::new(lx, y, 200.0, row_h), &mut max_f);
        graph.max_particles = max_f as usize;

        for conn in &graph.connections {
            let from = graph.nodes.iter().find(|n| n.id == conn.from_node);
            let to = graph.nodes.iter().find(|n| n.id == conn.to_node);
            if let (Some(fn_), Some(tn_)) = (from, to) {
                let fx = canvas_area.x + fn_.position[0] * self.zoom + self.scroll_offset[0] + node_w;
                let fy = canvas_area.y + fn_.position[1] * self.zoom + self.scroll_offset[1] + node_h * 0.5;
                let tx2 = canvas_area.x + tn_.position[0] * self.zoom + self.scroll_offset[0];
                let ty2 = canvas_area.y + tn_.position[1] * self.zoom + self.scroll_offset[1] + node_h * 0.5;
                ui.draw_list.line(Vec2::new(fx, fy), Vec2::new(tx2, ty2), p.accent, 1.5);
            }
        }

        let bottom_y = canvas_area.y + canvas_area.height - row_h - sp;
        let add_lbl_arr = ["+ Spawn", "+ Velocity", "+ Color", "+ Turb.", "+ Sprite"];
        let add_w = (w - pad * 2.0) / add_lbl_arr.len() as f64;
        for (k, lbl) in add_lbl_arr.iter().enumerate() {
            let btn_r = Rect::new(lx + k as f64 * add_w, bottom_y, add_w - 4.0, row_h);
            if Button::new(lbl).with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("add{k}")), btn_r).clicked {
                let nid = graph.nodes.len() as u32 + 10;
                let mut new_node = match k {
                    0 => VfxNode::spawn_rate(nid),
                    1 => VfxNode::initial_velocity(nid),
                    2 => VfxNode::color_over_lifetime(nid),
                    3 => VfxNode::turbulence(nid),
                    _ => VfxNode::sprite_renderer(nid),
                };
                new_node.position = [k as f64 * 180.0, 200.0 + graph.nodes.len() as f64 * 10.0];
                graph.add_node(new_node);
            }
        }
    }
}

fn cat_color(cat: &VfxNodeCategory, fallback: [f64; 4]) -> [f64; 4] {
    match cat {
        VfxNodeCategory::Emitter => [0.2, 0.5, 0.8, 1.0],
        VfxNodeCategory::Update => [0.2, 0.7, 0.3, 1.0],
        VfxNodeCategory::Render => [0.7, 0.3, 0.6, 1.0],
        VfxNodeCategory::Force => [0.8, 0.5, 0.1, 1.0],
        VfxNodeCategory::Noise => [0.5, 0.3, 0.8, 1.0],
        VfxNodeCategory::Event => [0.7, 0.2, 0.2, 1.0],
        VfxNodeCategory::Math => fallback,
    }
}

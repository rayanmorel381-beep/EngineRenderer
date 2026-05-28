use crate::scene::node_graph::{NodeGraph, ScriptNode};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown};

pub struct NodeEditorView {
    pub scroll_offset: [f64; 2],
    pub zoom: f64,
    pub new_node_kind: usize,
    pub pending_connect_from: Option<(usize, usize)>,
}

impl Default for NodeEditorView {
    fn default() -> Self { Self { scroll_offset: [0.0; 2], zoom: 1.0, new_node_kind: 0, pending_connect_from: None } }
}

impl NodeEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, graph: &mut NodeGraph) {
        let title = format!("Scripting visuel — {}", graph.name);
        let panel = Panel::new(&title).with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("node_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        let node_kinds = ["Event Tick", "Event BeginPlay", "Float Add", "Set Position", "Branch", "Print String"];
        Dropdown::new("Nouveau nœud", &node_kinds).show(ui, id.child("nk"), Rect::new(lx, y, w - 110.0 - sp, row_h), &mut self.new_node_kind);
        if Button::new("+ Ajouter").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("addnode"), Rect::new(lx + w - 110.0, y, 110.0, row_h)).clicked {
            let nid = graph.next_id;
            graph.next_id += 1;
            let node = match self.new_node_kind {
                0 => ScriptNode::event_tick(nid),
                1 => ScriptNode::event_begin_play(nid),
                2 => ScriptNode::float_add(nid),
                3 => ScriptNode::set_position(nid),
                4 => ScriptNode::branch(nid),
                _ => ScriptNode::print_string(nid),
            };
            graph.add_node(node);
        }
        y += row_h + sp;

        let canvas_rect = Rect::new(lx, y, w, body.height - (y - body.y) - pad);
        ui.draw_list.rect(canvas_rect, p.viewport_clear, 0.0);

        let node_w = 160.0;
        let node_h = 80.0;
        let mut remove_node: Option<usize> = None;

        for node in graph.nodes.iter() {
            let nx = canvas_rect.x + node.position[0] * self.zoom + self.scroll_offset[0];
            let ny = canvas_rect.y + node.position[1] * self.zoom + self.scroll_offset[1];
            let nr = Rect::new(nx, ny, node_w, node_h);
            if nr.x > canvas_rect.x + canvas_rect.width || nr.y > canvas_rect.y + canvas_rect.height { continue; }

            ui.draw_list.rect(nr, p.panel, 4.0);
            ui.draw_list.rect_outline(nr, p.border, 4.0, 1.0);
            ui.draw_list.text(Vec2::new(nx + 6.0, ny + 6.0), &node.title, p.text, m.font_size_normal);
            ui.draw_list.text(Vec2::new(nx + 6.0, ny + 6.0 + m.font_size_normal + 2.0), &node.category, p.text_muted, m.font_size_small);

            let del_r = Rect::new(nx + node_w - 20.0, ny + 2.0, 18.0, 18.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("ndel{}", node.id)), del_r).clicked {
                remove_node = Some(node.id);
            }

            for (pi, pin) in node.outputs.iter().enumerate() {
                let pin_y = ny + 30.0 + pi as f64 * 16.0;
                let color = pin.kind.color();
                let pin_col = [
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    255,
                ];
                let col_f = [pin_col[0] as f64 / 255.0, pin_col[1] as f64 / 255.0, pin_col[2] as f64 / 255.0, 1.0];
                ui.draw_list.text(Vec2::new(nx + node_w - 60.0, pin_y), &pin.label, col_f, m.font_size_small);
            }
            for (pi, pin) in node.inputs.iter().enumerate() {
                let pin_y = ny + 30.0 + pi as f64 * 16.0;
                let color = pin.kind.color();
                let pin_col = [
                    (color[0] * 255.0) as u8,
                    (color[1] * 255.0) as u8,
                    (color[2] * 255.0) as u8,
                    255,
                ];
                let col_f = [pin_col[0] as f64 / 255.0, pin_col[1] as f64 / 255.0, pin_col[2] as f64 / 255.0, 1.0];
                ui.draw_list.text(Vec2::new(nx + 6.0, pin_y), &pin.label, col_f, m.font_size_small);
            }
        }

        for conn in &graph.connections {
            let from_node = graph.nodes.iter().find(|n| n.id == conn.from_node);
            let to_node = graph.nodes.iter().find(|n| n.id == conn.to_node);
            if let (Some(fn_), Some(tn_)) = (from_node, to_node) {
                let fx = canvas_rect.x + fn_.position[0] * self.zoom + self.scroll_offset[0] + node_w;
                let fy = canvas_rect.y + fn_.position[1] * self.zoom + self.scroll_offset[1] + 30.0 + conn.from_pin as f64 * 16.0;
                let tx = canvas_rect.x + tn_.position[0] * self.zoom + self.scroll_offset[0];
                let ty = canvas_rect.y + tn_.position[1] * self.zoom + self.scroll_offset[1] + 30.0 + conn.to_pin as f64 * 16.0;
                ui.draw_list.line(Vec2::new(fx, fy), Vec2::new(tx, ty), p.accent, 1.5);
            }
        }

        if let Some(nid) = remove_node { graph.remove_node(nid); }

        let info = format!("Nœuds: {} | Connexions: {}", graph.nodes.len(), graph.connections.len());
        ui.draw_list.text(Vec2::new(lx, canvas_rect.y + canvas_rect.height + 2.0), &info, p.text_muted, m.font_size_small);
    }
}

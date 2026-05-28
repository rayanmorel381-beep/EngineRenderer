use crate::scene::pcg::{PcgGraph, PcgNodeKind};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Slider};

pub struct PcgEditorView {
    pub selected_node: Option<u32>,
    pub scroll_offset: [f64; 2],
    pub kind_idx: usize,
}

impl Default for PcgEditorView {
    fn default() -> Self { Self { selected_node: None, scroll_offset: [0.0; 2], kind_idx: 0 } }
}

impl PcgEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, graph: &mut PcgGraph) {
        let title = format!("PCG — {}", graph.name);
        let panel = Panel::new(&title).with_icon(Icon::PCG);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("pcg_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;

        let (sidebar_r, canvas_r) = body.split_left(180.0);
        let slx = sidebar_r.x + pad;
        let sw = sidebar_r.width - pad * 2.0;
        let mut sy = sidebar_r.y + pad;

        ui.draw_list.text(Vec2::new(slx, sy), &format!("Nœuds ({})", graph.nodes.len()), p.text, m.font_size_normal);
        sy += m.font_size_normal + sp;

        let mut remove_node: Option<u32> = None;
        for node in &graph.nodes {
            let sel = self.selected_node == Some(node.id);
            let nr = Rect::new(slx, sy, sw, row_h);
            ui.draw_list.rect(nr, if sel { p.panel_active } else { p.panel }, 2.0);
            let lbl = format!("{}: {}", node.id, node.kind.label());
            ui.draw_list.text(Vec2::new(slx + 4.0, sy + (row_h - m.font_size_small) * 0.5), &lbl, p.text, m.font_size_small);
            let del_r = Rect::new(slx + sw - 20.0, sy + 2.0, 18.0, row_h - 4.0);
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("ndel{}", node.id)), del_r).clicked {
                remove_node = Some(node.id);
            }
            if ui.is_rect_hovered(nr) { self.selected_node = Some(node.id); }
            sy += row_h + 2.0;
        }
        if let Some(nid) = remove_node { graph.remove_node(nid); self.selected_node = None; }

        let kind_lbl = PcgNodeKind::ALL[self.kind_idx % PcgNodeKind::ALL.len()].label();
        if Button::new(kind_lbl).with_style(ButtonStyle::Secondary).show(ui, id.child("kind_cyc"), Rect::new(slx, sy, sw, row_h)).clicked {
            self.kind_idx = (self.kind_idx + 1) % PcgNodeKind::ALL.len();
        }
        sy += row_h + 2.0;
        if Button::new("+ Nœud").with_style(ButtonStyle::Secondary).show(ui, id.child("add_nd"), Rect::new(slx, sy, sw, row_h)).clicked {
            let kind = PcgNodeKind::ALL[self.kind_idx % PcgNodeKind::ALL.len()].clone();
            graph.add_node(kind);
        }
        sy += row_h + 2.0;
        if let Some(sel_id) = self.selected_node {
            if let Some(node) = graph.nodes.iter_mut().find(|n| n.id == sel_id) {
                let mut seed_f = node.seed as f64;
                Slider::new("Graine", 0.0, 9999.0).show(ui, id.child("seed"), Rect::new(slx, sy, sw, row_h), &mut seed_f);
                node.seed = seed_f as u64;
            }
        }

        ui.draw_list.rect(canvas_r, p.viewport_clear, 0.0);
        let node_w = 140.0;
        let node_h = 70.0;
        for node in &graph.nodes {
            let nx = canvas_r.x + node.position[0] + self.scroll_offset[0];
            let ny = canvas_r.y + node.position[1] + self.scroll_offset[1];
            if nx + node_w < canvas_r.x || nx > canvas_r.x + canvas_r.width { continue; }
            let nr = Rect::new(nx, ny, node_w, node_h);
            let border = if self.selected_node == Some(node.id) { p.accent } else { p.border };
            ui.draw_list.rect(nr, p.panel, 4.0);
            ui.draw_list.rect_outline(nr, border, 4.0, 1.5);
            ui.draw_list.text(Vec2::new(nx + 4.0, ny + 6.0), node.kind.label(), p.text, m.font_size_small);
            let id_lbl = format!("#{}", node.id);
            ui.draw_list.text(Vec2::new(nx + 4.0, ny + 22.0), &id_lbl, p.text_muted, m.font_size_small);
        }

        for edge in &graph.edges {
            let from_node = graph.nodes.iter().find(|n| n.id == edge.from);
            let to_node = graph.nodes.iter().find(|n| n.id == edge.to);
            if let (Some(fn_), Some(tn_)) = (from_node, to_node) {
                let fx = canvas_r.x + fn_.position[0] + self.scroll_offset[0] + node_w;
                let fy = canvas_r.y + fn_.position[1] + self.scroll_offset[1] + node_h * 0.5;
                let tx2 = canvas_r.x + tn_.position[0] + self.scroll_offset[0];
                let ty2 = canvas_r.y + tn_.position[1] + self.scroll_offset[1] + node_h * 0.5;
                ui.draw_list.line(Vec2::new(fx, fy), Vec2::new(tx2, ty2), p.accent, 1.5);
            }
        }
        let y = body.y + body.height - m.font_size_small - sp;
        let footer = format!("{} nœuds | {} arêtes | Auto: {}", graph.nodes.len(), graph.edges.len(), graph.auto_execute);
        ui.draw_list.line(Vec2::new(lx, y - 2.0), Vec2::new(lx + w, y - 2.0), p.border, 1.0);
        ui.draw_list.text(Vec2::new(lx, y), &footer, p.text_muted, m.font_size_small);
    }
}

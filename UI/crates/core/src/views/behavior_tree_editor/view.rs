use crate::scene::behavior_tree::{BehaviorTree, BtNodeKind, ConditionOp};
use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Button, ButtonStyle, Dropdown};

pub struct BehaviorTreeEditorView {
    pub selected_node: Option<usize>,
    pub new_kind_idx: usize,
}

impl Default for BehaviorTreeEditorView {
    fn default() -> Self { Self { selected_node: None, new_kind_idx: 0 } }
}

impl BehaviorTreeEditorView {
    pub fn new() -> Self { Self::default() }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect, bt: &mut BehaviorTree) {
        let panel = Panel::new("Behavior Tree").with_icon(Icon::Settings);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);
        let m = ui.theme.metrics;
        let p = ui.theme.palette;
        let pad = m.padding;
        let row_h = m.row_height;
        let sp = m.spacing;
        let id = WidgetId::hash_str("bt_ed");
        let lx = body.x + pad;
        let w = body.width - pad * 2.0;
        let mut y = body.y + pad;

        ui.draw_list.text(Vec2::new(lx, y), &format!("Nœuds ({})", bt.nodes.len()), p.text, m.font_size_normal);
        y += m.font_size_normal + sp;

        let node_kinds = ["Sequence", "Selector", "Parallel", "Inverter", "Leaf (Action)", "Condition"];
        Dropdown::new("Type nœud", &node_kinds).show(ui, id.child("kind"), Rect::new(lx, y, w, row_h), &mut self.new_kind_idx);
        y += row_h + sp;

        if Button::new("+ Ajouter nœud").with_style(ButtonStyle::Secondary)
            .show(ui, id.child("add"), Rect::new(lx, y, w, row_h)).clicked {
            let kind = match self.new_kind_idx {
                0 => BtNodeKind::Sequence,
                1 => BtNodeKind::Selector,
                2 => BtNodeKind::Parallel,
                3 => BtNodeKind::Inverter,
                4 => BtNodeKind::Leaf { action: "NouvAction".into() },
                _ => BtNodeKind::Condition { key: "param".into(), op: ConditionOp::Greater, value_float: 0.0 },
            };
            bt.add_node(kind);
        }
        y += row_h + sp;

        let mut remove_node: Option<usize> = None;
        let nodes_snapshot: Vec<(usize, String, bool)> = bt.nodes.iter()
            .map(|n| (n.id, n.label.clone(), n.children.is_empty()))
            .collect();

        for (i, (nid, label, is_leaf)) in nodes_snapshot.iter().enumerate() {
            let row = Rect::new(lx, y, w, row_h);
            let is_sel = self.selected_node == Some(*nid);
            ui.draw_list.rect(row, if is_sel { p.selection } else { p.panel }, 2.0);
            let type_lbl = if *is_leaf { "◆" } else { "►" };
            let full_label = format!("{type_lbl} [{i}] {label}");
            ui.draw_list.text(Vec2::new(lx + 4.0, y + (row_h - m.font_size_normal) * 0.5), &full_label, p.text, m.font_size_normal);
            let sel_r = Rect::new(lx + w - 68.0, y + 2.0, 32.0, row_h - 4.0);
            let del_r = Rect::new(lx + w - 32.0, y + 2.0, 32.0, row_h - 4.0);
            if Button::new("✓").with_style(ButtonStyle::Secondary).show(ui, id.child(&format!("nsel{nid}")), sel_r).clicked {
                self.selected_node = if is_sel { None } else { Some(*nid) };
            }
            if Button::new("×").with_style(ButtonStyle::Danger).show(ui, id.child(&format!("ndel{nid}")), del_r).clicked {
                remove_node = Some(*nid);
            }
            y += row_h + 2.0;
        }
        if let Some(nid) = remove_node {
            bt.nodes.retain(|n| n.id != nid);
            bt.nodes.iter_mut().for_each(|n| n.children.retain(|&c| c != nid));
            if self.selected_node == Some(nid) { self.selected_node = None; }
        }

        y += sp;
        ui.draw_list.text(Vec2::new(lx, y), "Blackboard", p.text, m.font_size_normal);
        y += m.font_size_normal + sp;
        for (k, v) in bt.blackboard.entries.iter() {
            let val_str = format!("{k}: {v:?}");
            ui.draw_list.text(Vec2::new(lx + 8.0, y), &val_str, p.text_muted, m.font_size_small);
            y += m.font_size_small + 2.0;
        }
    }
}

use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::Rect;
use crate::ui::panels::Panel;
use crate::ui::style::icons::Icon;
use crate::ui::widgets::{Tree, TreeNode};

#[derive(Default)]
pub struct HierarchyView {
    pub nodes: Vec<TreeNode>,
    pub selected: Option<WidgetId>,
}

impl HierarchyView {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn replace_nodes(&mut self, nodes: Vec<TreeNode>) {
        self.nodes = nodes;
    }

    pub fn show(&mut self, ui: &mut UiContext, rect: Rect) {
        let panel = Panel::new("Hierarchy").with_icon(Icon::Scene);
        panel.show_chrome(ui, rect);
        let body = panel.body_rect(ui, rect);

        if let Some(id) = Tree::new(&mut self.nodes).show(ui, body) {
            self.selected = Some(id);
            for node in &mut self.nodes {
                node.selected = node.id == id;
            }
        }
    }
}

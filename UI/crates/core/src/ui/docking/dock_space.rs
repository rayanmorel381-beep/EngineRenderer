use crate::ui::docking::dock_node::DockNode;
use crate::ui::docking::drag_drop::DragDropState;
use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::Rect;

pub struct DockSpace {
    pub root: DockNode,
    pub drag: DragDropState,
}

impl DockSpace {
    pub fn new(root: DockNode) -> Self {
        Self {
            root,
            drag: DragDropState::default(),
        }
    }

    pub fn resize(&mut self, rect: Rect) {
        self.root.set_rect(rect);
    }

    pub fn render_chrome(&self, ui: &mut UiContext) {
        self.root.for_each_leaf(&mut |rect, _panels, _active| {
            ui.draw_list.rect(rect, ui.theme.palette.background, 0.0);
            ui.draw_list.rect_outline(
                rect,
                ui.theme.palette.border,
                ui.theme.metrics.border_width,
                0.0,
            );
        });
    }
}

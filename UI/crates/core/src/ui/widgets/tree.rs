use crate::ui::immediate::context::UiContext;
use crate::ui::immediate::id::WidgetId;
use crate::ui::layout::rect::{Rect, Vec2};
use crate::ui::style::icons::Icon;

pub struct TreeNode {
    pub id: WidgetId,
    pub label: String,
    pub icon: Icon,
    pub depth: usize,
    pub expanded: bool,
    pub selected: bool,
    pub has_children: bool,
}

impl TreeNode {
    pub fn new(id: WidgetId, label: impl Into<String>, depth: usize) -> Self {
        Self {
            id,
            label: label.into(),
            icon: Icon::None,
            depth,
            expanded: true,
            selected: false,
            has_children: false,
        }
    }
}

pub struct Tree<'a> {
    pub nodes: &'a mut [TreeNode],
    pub row_height: f64,
}

impl<'a> Tree<'a> {
    pub fn new(nodes: &'a mut [TreeNode]) -> Self {
        Self {
            nodes,
            row_height: 22.0,
        }
    }

    pub fn show(self, ui: &mut UiContext, rect: Rect) -> Option<WidgetId> {
        let metrics = ui.theme.metrics;
        let palette = ui.theme.palette;
        let pointer = Vec2::new(ui.input.pointer.x, ui.input.pointer.y);
        let mut clicked: Option<WidgetId> = None;
        let mut y = rect.y;

        for node in self.nodes.iter_mut() {
            if y + self.row_height > rect.y + rect.height {
                break;
            }
            let row = Rect::new(rect.x, y, rect.width, self.row_height);
            let hovered = row.contains(pointer);

            if hovered && ui.input.pointer.left_down && ui.active != node.id {
                ui.set_active(node.id);
                clicked = Some(node.id);
                node.selected = true;
                if node.has_children {
                    node.expanded = !node.expanded;
                }
            }
            if !ui.input.pointer.left_down && ui.active == node.id {
                ui.clear_active();
            }

            let bg = if node.selected {
                palette.selection
            } else if hovered {
                palette.panel_hover
            } else {
                [0.0, 0.0, 0.0, 0.0]
            };
            if bg[3] > 0.0 {
                ui.draw_list.rect(row, bg, 0.0);
            }

            let indent = node.depth as f64 * metrics.indent_step;
            let chevron = if node.has_children {
                if node.expanded {
                    Icon::ChevronDown.glyph()
                } else {
                    Icon::ChevronRight.glyph()
                }
            } else {
                " "
            };
            let icon = node.icon.glyph();
            let text = format!("{} {} {}", chevron, icon, node.label);
            ui.draw_list.text(
                Vec2::new(
                    row.x + metrics.padding_small + indent,
                    row.y + (row.height - metrics.font_size_normal) * 0.5,
                ),
                text,
                palette.text,
                metrics.font_size_normal,
            );

            y += self.row_height;
        }

        clicked
    }
}

use crate::ui::layout::rect::Rect;

pub struct EditorLayout {
    pub menu_bar: Rect,
    pub tool_bar: Rect,
    pub status_bar: Rect,
    pub left_panel: Rect,
    pub right_panel: Rect,
    pub bottom_panel: Rect,
    pub center_panel: Rect,
}

impl EditorLayout {
    pub fn compute(viewport: Rect, menu_h: f64, tool_h: f64, status_h: f64) -> Self {
        let (menu_bar, after_menu) = viewport.split_top(menu_h);
        let (tool_bar, after_tool) = after_menu.split_top(tool_h);
        let (working_area, status_bar) = after_tool.split_bottom(status_h);

        let min_center_w = (working_area.width * 0.42).clamp(220.0, working_area.width);
        let side_budget = (working_area.width - min_center_w).max(0.0);
        let left_w = (working_area.width * 0.18).min(side_budget * 0.48);
        let right_w = (working_area.width * 0.22).min((side_budget - left_w).max(0.0));
        let min_center_h = (working_area.height * 0.42).clamp(160.0, working_area.height);
        let bottom_budget = (working_area.height - min_center_h).max(0.0);
        let bottom_h = (working_area.height * 0.28).min(bottom_budget);

        let (left_panel, after_left) = working_area.split_left(left_w);
        let (after_right_split, right_panel) = after_left.split_right(right_w);
        let (center_panel, bottom_panel) = after_right_split.split_bottom(bottom_h);

        Self {
            menu_bar,
            tool_bar,
            status_bar,
            left_panel,
            right_panel,
            bottom_panel,
            center_panel,
        }
    }
}

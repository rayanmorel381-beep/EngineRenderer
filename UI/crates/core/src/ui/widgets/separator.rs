use crate::ui::immediate::context::UiContext;
use crate::ui::layout::rect::{Rect, Vec2};

pub struct Separator;

impl Separator {
    pub fn horizontal(ui: &mut UiContext, rect: Rect) {
        let y = rect.y + rect.height * 0.5;
        ui.draw_list.line(
            Vec2::new(rect.x, y),
            Vec2::new(rect.x + rect.width, y),
            ui.theme.palette.border,
            ui.theme.metrics.border_width,
        );
    }

    pub fn vertical(ui: &mut UiContext, rect: Rect) {
        let x = rect.x + rect.width * 0.5;
        ui.draw_list.line(
            Vec2::new(x, rect.y),
            Vec2::new(x, rect.y + rect.height),
            ui.theme.palette.border,
            ui.theme.metrics.border_width,
        );
    }
}

use crate::ui::immediate::draw_list::DrawList;
use crate::ui::immediate::id::WidgetId;
use crate::ui::input::input_state::InputState;
use crate::ui::layout::rect::Rect;
use crate::ui::style::theme::Theme;

pub struct UiContext {
    pub viewport_w: u32,
    pub viewport_h: u32,
    pub theme: Theme,
    pub input: InputState,
    pub draw_list: DrawList,
    pub hovered: WidgetId,
    pub active: WidgetId,
    pub focused: WidgetId,
    pub frame_index: u64,
    pub time_seconds: f64,
}

impl UiContext {
    pub fn new(viewport_w: u32, viewport_h: u32) -> Self {
        Self {
            viewport_w,
            viewport_h,
            theme: Theme::DARK,
            input: InputState::new(),
            draw_list: DrawList::new(),
            hovered: WidgetId::NONE,
            active: WidgetId::NONE,
            focused: WidgetId::NONE,
            frame_index: 0,
            time_seconds: 0.0,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn screen_rect(&self) -> Rect {
        Rect::new(0.0, 0.0, self.viewport_w as f64, self.viewport_h as f64)
    }

    pub fn begin_frame(&mut self, frame_index: u64, time_seconds: f64) {
        self.draw_list.clear();
        self.hovered = WidgetId::NONE;
        self.frame_index = frame_index;
        self.time_seconds = time_seconds;
    }

    pub fn end_frame(&mut self) {
        self.input.end_frame();
    }

    pub fn set_hovered(&mut self, id: WidgetId) {
        self.hovered = id;
    }

    pub fn set_active(&mut self, id: WidgetId) {
        self.active = id;
    }

    pub fn clear_active(&mut self) {
        self.active = WidgetId::NONE;
    }

    pub fn is_rect_hovered(&self, rect: crate::ui::layout::rect::Rect) -> bool {
        let mx = self.input.pointer.x;
        let my = self.input.pointer.y;
        mx >= rect.x && mx <= rect.x + rect.width && my >= rect.y && my <= rect.y + rect.height
    }

    pub fn focus(&mut self, id: WidgetId) {
        self.focused = id;
    }
}

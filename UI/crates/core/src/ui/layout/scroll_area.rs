use crate::ui::layout::rect::Rect;

pub struct ScrollArea {
    pub offset_x: f64,
    pub offset_y: f64,
    pub content_width: f64,
    pub content_height: f64,
    pub scroll_bar_width: f64,
    pub horizontal_enabled: bool,
    pub vertical_enabled: bool,
}

impl ScrollArea {
    pub fn new() -> Self {
        Self {
            offset_x: 0.0,
            offset_y: 0.0,
            content_width: 0.0,
            content_height: 0.0,
            scroll_bar_width: 12.0,
            horizontal_enabled: true,
            vertical_enabled: true,
        }
    }

    pub fn with_content(mut self, width: f64, height: f64) -> Self {
        self.content_width = width;
        self.content_height = height;
        self
    }

    pub fn viewport(&self, container: Rect) -> Rect {
        let mut viewport = container;
        if self.vertical_enabled && self.content_height > viewport.height {
            viewport.width = (viewport.width - self.scroll_bar_width).max(0.0);
        }
        if self.horizontal_enabled && self.content_width > viewport.width {
            viewport.height = (viewport.height - self.scroll_bar_width).max(0.0);
        }
        viewport
    }

    pub fn vertical_bar(&self, container: Rect) -> Option<Rect> {
        if !self.vertical_enabled || self.content_height <= container.height {
            return None;
        }
        Some(Rect::new(
            container.x + container.width - self.scroll_bar_width,
            container.y,
            self.scroll_bar_width,
            container.height,
        ))
    }

    pub fn horizontal_bar(&self, container: Rect) -> Option<Rect> {
        if !self.horizontal_enabled || self.content_width <= container.width {
            return None;
        }
        Some(Rect::new(
            container.x,
            container.y + container.height - self.scroll_bar_width,
            container.width,
            self.scroll_bar_width,
        ))
    }

    pub fn scroll_by(&mut self, dx: f64, dy: f64, viewport: Rect) {
        self.offset_x = (self.offset_x + dx)
            .clamp(0.0, (self.content_width - viewport.width).max(0.0));
        self.offset_y = (self.offset_y + dy)
            .clamp(0.0, (self.content_height - viewport.height).max(0.0));
    }

    pub fn content_origin(&self, viewport: Rect) -> (f64, f64) {
        (viewport.x - self.offset_x, viewport.y - self.offset_y)
    }
}

impl Default for ScrollArea {
    fn default() -> Self {
        Self::new()
    }
}

use crate::ui::layout::rect::Rect;

pub struct Stack {
    pub padding: f64,
}

impl Stack {
    pub fn new() -> Self {
        Self { padding: 0.0 }
    }

    pub fn with_padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }

    pub fn area(&self, container: Rect) -> Rect {
        container.shrink(self.padding)
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self::new()
    }
}

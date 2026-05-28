#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    Other(u8),
}

impl MouseButton {
    pub fn from_raw(button: u32) -> Self {
        match button {
            1 => MouseButton::Left,
            2 => MouseButton::Middle,
            3 => MouseButton::Right,
            other => MouseButton::Other(other.min(255) as u8),
        }
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct PointerState {
    pub x: f64,
    pub y: f64,
    pub last_x: f64,
    pub last_y: f64,
    pub left_down: bool,
    pub right_down: bool,
    pub middle_down: bool,
    pub scroll_x: f64,
    pub scroll_y: f64,
}

impl PointerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delta(&self) -> (f64, f64) {
        (self.x - self.last_x, self.y - self.last_y)
    }

    pub fn end_frame(&mut self) {
        self.last_x = self.x;
        self.last_y = self.y;
        self.scroll_x = 0.0;
        self.scroll_y = 0.0;
    }

    pub fn is_button_down(&self, button: MouseButton) -> bool {
        match button {
            MouseButton::Left => self.left_down,
            MouseButton::Right => self.right_down,
            MouseButton::Middle => self.middle_down,
            MouseButton::Other(_) => false,
        }
    }

    pub fn set_button(&mut self, button: MouseButton, down: bool) {
        match button {
            MouseButton::Left => self.left_down = down,
            MouseButton::Right => self.right_down = down,
            MouseButton::Middle => self.middle_down = down,
            MouseButton::Other(_) => {}
        }
    }
}

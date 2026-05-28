use crate::ui::input::keyboard::KeyboardState;
use crate::ui::input::modifiers::Modifiers;
use crate::ui::input::pointer::PointerState;

#[derive(Default)]
pub struct InputState {
    pub pointer: PointerState,
    pub keyboard: KeyboardState,
    pub modifiers: Modifiers,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn end_frame(&mut self) {
        self.pointer.end_frame();
        self.keyboard.drain_typed();
    }
}

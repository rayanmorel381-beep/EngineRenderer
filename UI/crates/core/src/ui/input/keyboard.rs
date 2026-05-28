#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Key(pub u32);

impl Key {
    pub const ESCAPE: Self = Self(0xFF1B);
    pub const ENTER: Self = Self(0xFF0D);
    pub const TAB: Self = Self(0xFF09);
    pub const BACKSPACE: Self = Self(0xFF08);
    pub const DELETE: Self = Self(0xFFFF);
    pub const LEFT: Self = Self(0xFF51);
    pub const UP: Self = Self(0xFF52);
    pub const RIGHT: Self = Self(0xFF53);
    pub const DOWN: Self = Self(0xFF54);
    pub const F1: Self = Self(0xFFBE);
    pub const F12: Self = Self(0xFFC9);
    pub const SPACE: Self = Self(0x0020);
    pub const SHIFT_L: Self = Self(0xFFE1);
    pub const SHIFT_R: Self = Self(0xFFE2);
    pub const CTRL_L: Self = Self(0xFFE3);
    pub const CTRL_R: Self = Self(0xFFE4);
    pub const ALT_L: Self = Self(0xFFE9);
    pub const ALT_R: Self = Self(0xFFEA);
    pub const META_L: Self = Self(0xFFE7);
    pub const META_R: Self = Self(0xFFE8);
    pub const SUPER_L: Self = Self(0xFFEB);
    pub const SUPER_R: Self = Self(0xFFEC);
    pub const Q: Self = Self(0x0071);
    pub const S: Self = Self(0x0073);
    pub const Z: Self = Self(0x007A);
    pub const Y: Self = Self(0x0079);
    pub const N: Self = Self(0x006E);
    pub const O: Self = Self(0x006F);
    pub const E: Self = Self(0x0065);
    pub const G: Self = Self(0x0067);
    pub const R: Self = Self(0x0072);
    pub const F: Self = Self(0x0066);
    pub const D: Self = Self(0x0064);
    pub const A: Self = Self(0x0061);
    pub const H: Self = Self(0x0068);
    pub const NUMPAD_5: Self = Self(0xFFB5);
    pub const PERIOD: Self = Self(0x002E);
}

#[derive(Default)]
pub struct KeyboardState {
    pressed: Vec<Key>,
    typed: String,
}

impl KeyboardState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn press(&mut self, key: Key) {
        if !self.pressed.contains(&key) {
            self.pressed.push(key);
        }
    }

    pub fn release(&mut self, key: Key) {
        self.pressed.retain(|k| *k != key);
    }

    pub fn type_text(&mut self, ch: char) {
        self.typed.push(ch);
    }

    pub fn is_down(&self, key: Key) -> bool {
        self.pressed.contains(&key)
    }

    pub fn drain_typed(&mut self) -> String {
        std::mem::take(&mut self.typed)
    }
}

use crate::state::AppState;
use crate::ui::immediate::context::UiContext;
use crate::ui::input::keyboard::Key;
use crate::ui::input::modifiers::Modifiers;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShortcutAction {
    Quit,
    Save,
    Open,
    NewScene,
    Undo,
    Redo,
    DeleteSelected,
    DuplicateSelected,
    FrameSelected,
    ToggleEditMode,
    ExtrudeSelected,
    LoopCut,
    Subdivide,
}

#[derive(Copy, Clone, Debug)]
pub struct Shortcut {
    pub modifiers: Modifiers,
    pub key: Key,
    pub action: ShortcutAction,
}

pub fn default_bindings() -> Vec<Shortcut> {
    let ctrl = Modifiers {
        shift: false,
        ctrl: true,
        alt: false,
        meta: false,
    };
    let ctrl_shift = Modifiers {
        shift: true,
        ctrl: true,
        alt: false,
        meta: false,
    };
    vec![
        Shortcut { modifiers: ctrl, key: Key::Q, action: ShortcutAction::Quit },
        Shortcut { modifiers: ctrl, key: Key::S, action: ShortcutAction::Save },
        Shortcut { modifiers: ctrl, key: Key::O, action: ShortcutAction::Open },
        Shortcut { modifiers: ctrl, key: Key::N, action: ShortcutAction::NewScene },
        Shortcut { modifiers: ctrl, key: Key::Z, action: ShortcutAction::Undo },
        Shortcut { modifiers: ctrl_shift, key: Key::Z, action: ShortcutAction::Redo },
        Shortcut { modifiers: Modifiers::NONE, key: Key::DELETE, action: ShortcutAction::DeleteSelected },
        Shortcut { modifiers: ctrl, key: Key::D, action: ShortcutAction::DuplicateSelected },
        Shortcut { modifiers: Modifiers::NONE, key: Key::F, action: ShortcutAction::FrameSelected },
        Shortcut { modifiers: Modifiers::NONE, key: Key::TAB, action: ShortcutAction::ToggleEditMode },
        Shortcut { modifiers: Modifiers::NONE, key: Key::E, action: ShortcutAction::ExtrudeSelected },
        Shortcut { modifiers: ctrl, key: Key::R, action: ShortcutAction::LoopCut },
        Shortcut { modifiers: Modifiers::NONE, key: Key::NUMPAD_5, action: ShortcutAction::Subdivide },
    ]
}

#[derive(Default)]
pub struct ShortcutTracker {
    last_pressed: Vec<Key>,
}

impl ShortcutTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn poll(&mut self, ui: &UiContext, bindings: &[Shortcut]) -> Vec<ShortcutAction> {
        let mut fired = Vec::new();
        let mods = ui.input.modifiers;
        let pressed_now: Vec<Key> = bindings
            .iter()
            .filter_map(|s| {
                if mods == s.modifiers && ui.input.keyboard.is_down(s.key) {
                    Some(s.key)
                } else {
                    None
                }
            })
            .collect();
        for shortcut in bindings {
            if mods == shortcut.modifiers
                && pressed_now.contains(&shortcut.key)
                && !self.last_pressed.contains(&shortcut.key)
            {
                fired.push(shortcut.action);
            }
        }
        self.last_pressed = pressed_now;
        fired
    }
}

pub fn apply(action: ShortcutAction, state: &mut AppState) -> Option<ShortcutAction> {
    match action {
        ShortcutAction::Quit => {
            state.should_quit = true;
            None
        }
        other => Some(other),
    }
}

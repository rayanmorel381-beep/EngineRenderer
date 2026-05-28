use enginerenderer::api::display::BackendEvent;

use crate::state::AppState;
use crate::ui::immediate::UiContext;
use crate::ui::input::keyboard::Key;
use crate::ui::input::pointer::MouseButton;

pub fn dispatch(event: &BackendEvent, ui: &mut UiContext, state: &mut AppState) {
    match event {
        BackendEvent::CloseRequested => state.should_quit = true,
        BackendEvent::Resized { width, height } => {
            state.resize(*width as usize, *height as usize);
            ui.viewport_w = *width;
            ui.viewport_h = *height;
        }
        BackendEvent::MouseMove { x, y } => {
            ui.input.pointer.x = *x as f64;
            ui.input.pointer.y = *y as f64;
        }
        BackendEvent::MouseButtonPress { button, x, y } => {
            ui.input.pointer.x = *x as f64;
            ui.input.pointer.y = *y as f64;
            match *button {
                4 => ui.input.pointer.scroll_y += 1.0,
                5 => ui.input.pointer.scroll_y -= 1.0,
                _ => ui.input.pointer.set_button(MouseButton::from_raw(*button), true),
            }
        }
        BackendEvent::MouseButtonRelease { button, x, y } => {
            ui.input.pointer.x = *x as f64;
            ui.input.pointer.y = *y as f64;
            match *button {
                4 | 5 => {}
                _ => ui.input.pointer.set_button(MouseButton::from_raw(*button), false),
            }
        }
        BackendEvent::KeyPress { keysym } => {
            let key = Key(*keysym as u32);
            ui.input.keyboard.press(key);
            update_modifiers(&mut ui.input.modifiers, key, true);
            if key == Key::ESCAPE {
                state.should_quit = true;
            }
        }
        BackendEvent::KeyRelease { keysym } => {
            let key = Key(*keysym as u32);
            ui.input.keyboard.release(key);
            update_modifiers(&mut ui.input.modifiers, key, false);
        }
        _ => {}
    }
}

fn update_modifiers(modifiers: &mut crate::ui::input::modifiers::Modifiers, key: Key, pressed: bool) {
    match key {
        Key::SHIFT_L | Key::SHIFT_R => modifiers.shift = pressed,
        Key::CTRL_L | Key::CTRL_R => modifiers.ctrl = pressed,
        Key::ALT_L | Key::ALT_R => modifiers.alt = pressed,
        Key::META_L | Key::META_R | Key::SUPER_L | Key::SUPER_R => modifiers.meta = pressed,
        _ => {}
    }
}

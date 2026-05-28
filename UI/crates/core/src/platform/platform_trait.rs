use crate::config::AppConfig;
use crate::state::AppState;
use enginerenderer::api::display::NativeWindow;

pub trait Platform {
    fn config(&self) -> AppConfig;
    fn on_start(&mut self, window: &NativeWindow, state: &mut AppState);
    fn on_frame(&mut self, window: &NativeWindow, state: &mut AppState);
    fn on_shutdown(&mut self, state: &mut AppState);
}

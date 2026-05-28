use enginerenderer::api::display::NativeWindow;
use ruxel_core::{AppConfig, AppState, Platform, run};

pub struct IosPlatform;

impl Platform for IosPlatform {
    fn config(&self) -> AppConfig {
        AppConfig::new(1170, 2532, "EngineRenderer")
    }

    fn on_start(&mut self, window: &NativeWindow, state: &mut AppState) {
        window.make_current();
        state.frame_index = 0;
    }

    fn on_frame(&mut self, window: &NativeWindow, state: &mut AppState) {
        window.make_current();
        state.width = state.width.max(1);
    }

    fn on_shutdown(&mut self, state: &mut AppState) {
        state.should_quit = true;
    }
}

pub fn launch() -> i32 {
    run(IosPlatform)
}

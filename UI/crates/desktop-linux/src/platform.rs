use enginerenderer::api::display::NativeWindow;
use ruxel_core::{AppConfig, AppState, Platform, run};

pub struct LinuxPlatform;

impl Platform for LinuxPlatform {
    fn config(&self) -> AppConfig {
        AppConfig::new(1280, 720, "EngineRenderer")
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
    run(LinuxPlatform)
}

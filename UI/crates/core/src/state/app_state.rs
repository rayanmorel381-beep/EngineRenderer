
pub struct AppState {
    pub frame_index: u64,
    pub width: usize,
    pub height: usize,
    pub should_quit: bool,
    pub last_frame_micros: u64,
}

impl AppState {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            frame_index: 0,
            width,
            height,
            should_quit: false,
            last_frame_micros: 0,
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        self.width = width.max(1);
        self.height = height.max(1);
    }

    pub fn tick(&mut self, frame_micros: u64) {
        self.frame_index = self.frame_index.wrapping_add(1);
        self.last_frame_micros = frame_micros;
    }
}

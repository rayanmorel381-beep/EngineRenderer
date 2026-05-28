pub struct AppConfig {
    pub width: usize,
    pub height: usize,
    pub title: String,
}

impl AppConfig {
    pub fn new(width: usize, height: usize, title: impl Into<String>) -> Self {
        Self {
            width,
            height,
            title: title.into(),
        }
    }

    pub fn desktop_default(title: impl Into<String>) -> Self {
        Self::new(1280, 720, title)
    }

    pub fn mobile_default(title: impl Into<String>) -> Self {
        Self::new(1080, 1920, title)
    }
}

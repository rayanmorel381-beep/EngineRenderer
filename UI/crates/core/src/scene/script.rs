#[derive(Clone, Debug)]
pub struct ScriptComponent {
    pub name: String,
    pub source: String,
    pub enabled: bool,
}

impl ScriptComponent {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            source: String::new(),
            enabled: true,
        }
    }

    pub fn with_source(mut self, src: impl Into<String>) -> Self {
        self.source = src.into();
        self
    }

    pub fn on_update_stub(&self) -> String {
        format!(
            "-- Script: {}\n-- fn on_update(dt: f64) is called every frame\nfunction on_update(dt)\n    -- write logic here\nend\n",
            self.name
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StreamState {
    Unloaded,
    Requested,
    Loading,
    Loaded,
    Unloading,
}

impl StreamState {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unloaded => "Non chargé", Self::Requested => "Demandé",
            Self::Loading => "Chargement…", Self::Loaded => "Chargé", Self::Unloading => "Déchargement",
        }
    }
}

impl Default for StreamState { fn default() -> Self { Self::Unloaded } }

#[derive(Clone, Debug)]
pub struct StreamingLevel {
    pub name: String,
    pub path: String,
    pub state: StreamState,
    pub load_distance: f64,
    pub unload_distance: f64,
    pub reference_point: [f64; 3],
    pub priority: i32,
    pub always_loaded: bool,
    pub memory_mb: f64,
}

impl StreamingLevel {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            state: StreamState::Unloaded,
            load_distance: 200.0,
            unload_distance: 300.0,
            reference_point: [0.0; 3],
            priority: 0,
            always_loaded: false,
            memory_mb: 0.0,
        }
    }

    pub fn evaluate_from_camera(&mut self, camera_pos: [f64; 3]) {
        let dx = self.reference_point[0] - camera_pos[0];
        let dy = self.reference_point[1] - camera_pos[1];
        let dz = self.reference_point[2] - camera_pos[2];
        let dist = (dx*dx + dy*dy + dz*dz).sqrt();
        match self.state {
            StreamState::Unloaded | StreamState::Unloading => {
                if self.always_loaded || dist < self.load_distance { self.state = StreamState::Requested; }
            }
            StreamState::Loaded => {
                if !self.always_loaded && dist > self.unload_distance { self.state = StreamState::Unloading; }
            }
            _ => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct StreamingBudget {
    pub max_memory_mb: f64,
    pub used_memory_mb: f64,
    pub max_concurrent_loads: usize,
    pub current_loads: usize,
}

impl Default for StreamingBudget {
    fn default() -> Self {
        Self { max_memory_mb: 2048.0, used_memory_mb: 0.0, max_concurrent_loads: 4, current_loads: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct AssetStreamingConfig {
    pub enabled: bool,
    pub levels: Vec<StreamingLevel>,
    pub budget: StreamingBudget,
    pub distance_based: bool,
    pub tick_interval_s: f64,
}

impl Default for AssetStreamingConfig {
    fn default() -> Self {
        Self { enabled: true, levels: Vec::new(), budget: StreamingBudget::default(), distance_based: true, tick_interval_s: 0.5 }
    }
}

impl AssetStreamingConfig {
    pub fn new() -> Self { Self::default() }

    pub fn tick(&mut self, camera_pos: [f64; 3]) {
        if !self.enabled { return; }
        for level in self.levels.iter_mut() {
            level.evaluate_from_camera(camera_pos);
        }
    }

    pub fn loaded_count(&self) -> usize { self.levels.iter().filter(|l| l.state == StreamState::Loaded).count() }
    pub fn loading_count(&self) -> usize { self.levels.iter().filter(|l| l.state == StreamState::Loading).count() }
}

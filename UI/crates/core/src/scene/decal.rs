#[derive(Clone, Debug, PartialEq)]
pub enum DecalBlendMode {
    Multiply,
    Add,
    Normal,
    Albedo,
    Roughness,
}

impl DecalBlendMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Multiply => "Multiplier", Self::Add => "Additionner",
            Self::Normal => "Normal Map", Self::Albedo => "Albedo", Self::Roughness => "Rugosité",
        }
    }
    pub const ALL: [DecalBlendMode; 5] = [
        DecalBlendMode::Multiply, DecalBlendMode::Add, DecalBlendMode::Normal,
        DecalBlendMode::Albedo, DecalBlendMode::Roughness,
    ];
}

#[derive(Clone, Debug)]
pub struct Decal {
    pub name: String,
    pub texture_asset: Option<String>,
    pub normal_asset: Option<String>,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub size: [f64; 3],
    pub blend_mode: DecalBlendMode,
    pub opacity: f64,
    pub depth_fade: f64,
    pub angle_fade: f64,
    pub sort_order: i32,
    pub enabled: bool,
    pub fade_start: f64,
    pub fade_end: f64,
}

impl Default for Decal {
    fn default() -> Self {
        Self {
            name: "Décal".to_string(),
            texture_asset: None,
            normal_asset: None,
            position: [0.0; 3],
            rotation: [0.0; 3],
            size: [1.0, 1.0, 1.0],
            blend_mode: DecalBlendMode::Albedo,
            opacity: 1.0,
            depth_fade: 0.1,
            angle_fade: 80.0,
            sort_order: 0,
            enabled: true,
            fade_start: 10.0,
            fade_end: 20.0,
        }
    }
}

impl Decal {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    pub fn projects_on(&self, point: [f64; 3]) -> bool {
        let dx = (point[0] - self.position[0]).abs();
        let dy = (point[1] - self.position[1]).abs();
        let dz = (point[2] - self.position[2]).abs();
        dx < self.size[0] * 0.5 && dy < self.size[1] * 0.5 && dz < self.size[2] * 0.5
    }
}

#[derive(Clone, Debug, Default)]
pub struct DecalLayer {
    pub decals: Vec<Decal>,
}

impl DecalLayer {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, decal: Decal) { self.decals.push(decal); }

    pub fn remove(&mut self, index: usize) {
        if index < self.decals.len() { self.decals.remove(index); }
    }

    pub fn decals_at(&self, point: [f64; 3]) -> Vec<usize> {
        self.decals.iter().enumerate().filter_map(|(i, d)| if d.enabled && d.projects_on(point) { Some(i) } else { None }).collect()
    }
}

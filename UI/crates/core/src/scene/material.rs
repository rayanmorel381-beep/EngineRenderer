#[derive(Clone, Debug, PartialEq)]
pub enum AlphaMode {
    Opaque,
    Mask,
    Blend,
}

impl Default for AlphaMode {
    fn default() -> Self { Self::Opaque }
}

impl AlphaMode {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Opaque => "Opaque",
            Self::Mask => "Mask",
            Self::Blend => "Blend",
        }
    }
    pub const ALL: [AlphaMode; 3] = [AlphaMode::Opaque, AlphaMode::Mask, AlphaMode::Blend];
}

#[derive(Clone, Debug)]
pub struct TextureRef {
    pub asset_index: usize,
}

impl TextureRef {
    pub fn new(asset_index: usize) -> Self { Self { asset_index } }
}

#[derive(Clone, Debug)]
pub struct PbrMaterial {
    pub name: String,
    pub albedo: [f64; 4],
    pub metallic: f64,
    pub roughness: f64,
    pub emissive: [f64; 3],
    pub emissive_strength: f64,
    pub normal_scale: f64,
    pub occlusion_strength: f64,
    pub alpha_mode: AlphaMode,
    pub alpha_cutoff: f64,
    pub double_sided: bool,
    pub albedo_tex: Option<TextureRef>,
    pub metallic_roughness_tex: Option<TextureRef>,
    pub normal_tex: Option<TextureRef>,
    pub emissive_tex: Option<TextureRef>,
    pub occlusion_tex: Option<TextureRef>,
    pub cast_shadows: bool,
    pub receive_shadows: bool,
    pub wireframe: bool,
}

impl Default for PbrMaterial {
    fn default() -> Self {
        Self {
            name: "Material".to_string(),
            albedo: [0.8, 0.8, 0.8, 1.0],
            metallic: 0.0,
            roughness: 0.5,
            emissive: [0.0, 0.0, 0.0],
            emissive_strength: 1.0,
            normal_scale: 1.0,
            occlusion_strength: 1.0,
            alpha_mode: AlphaMode::Opaque,
            alpha_cutoff: 0.5,
            double_sided: false,
            albedo_tex: None,
            metallic_roughness_tex: None,
            normal_tex: None,
            emissive_tex: None,
            occlusion_tex: None,
            cast_shadows: true,
            receive_shadows: true,
            wireframe: false,
        }
    }
}

impl PbrMaterial {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }
    pub fn with_albedo(mut self, r: f64, g: f64, b: f64, a: f64) -> Self {
        self.albedo = [r, g, b, a]; self
    }
    pub fn with_metallic(mut self, v: f64) -> Self { self.metallic = v; self }
    pub fn with_roughness(mut self, v: f64) -> Self { self.roughness = v; self }
}

#[derive(Clone, Debug, Default)]
pub struct MaterialLibrary {
    pub materials: Vec<PbrMaterial>,
}

impl MaterialLibrary {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, mat: PbrMaterial) -> usize {
        let idx = self.materials.len();
        self.materials.push(mat);
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&PbrMaterial> { self.materials.get(idx) }
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut PbrMaterial> { self.materials.get_mut(idx) }
    pub fn len(&self) -> usize { self.materials.len() }
    pub fn is_empty(&self) -> bool { self.materials.is_empty() }
}

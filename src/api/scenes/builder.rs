use std::{fs, io, path::Path, str::FromStr};

use crate::api::materials::catalog::MaterialCatalog;
use crate::api::objects::SceneObject;
use crate::api::types::CameraDesc;
use crate::core::engine::rendering::raytracing::{
    AreaLight, Camera, DirectionalLight, Material, Scene, Sphere, Triangle, Vec3,
};
use crate::core::engine::rendering::effects::volumetric_effects::medium::VolumetricMedium;

/// Fluent scene construction helper for API consumers.
#[derive(Debug, Clone)]
pub struct SceneBuilder {
    spheres: Vec<Sphere>,
    triangles: Vec<Triangle>,
    sun_direction: Vec3,
    sun_color: Vec3,
    sun_intensity: f64,
    sun_angular_radius: f64,
    area_lights: Vec<AreaLight>,
    sky_top: Vec3,
    sky_bottom: Vec3,
    exposure: f64,
    volume: VolumetricMedium,
    camera: CameraDesc,
}

impl Default for SceneBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SceneBuilder {
    /// Creates an empty scene builder with cinematic defaults.
    pub fn new() -> Self {
        Self {
            spheres: Vec::new(),
            triangles: Vec::new(),
            sun_direction: Vec3::new(-0.65, -0.35, -1.0).normalize(),
            sun_color: Vec3::new(1.0, 0.96, 0.90),
            sun_intensity: 1.45,
            sun_angular_radius: 0.03,
            area_lights: Vec::new(),
            sky_top: Vec3::new(0.015, 0.020, 0.050),
            sky_bottom: Vec3::new(0.001, 0.001, 0.006),
            exposure: 1.45,
            volume: VolumetricMedium::cinematic_nebula().with_density_multiplier(0.9),
            camera: CameraDesc::default(),
        }
    }

    // -----------------------------------------------------------------------
    // Objects
    // -----------------------------------------------------------------------

    /// Adds a sphere object.
    pub fn add_sphere(mut self, center: Vec3, radius: f64, material: Material) -> Self {
        self.spheres.push(Sphere {
            center,
            radius: radius.max(0.01),
            material,
        });
        self
    }

    /// Adds a sphere object using a material name from the catalog.
    pub fn add_sphere_named(self, center: Vec3, radius: f64, material_name: &str) -> Self {
        let material = MaterialCatalog.by_name(material_name);
        self.add_sphere(center, radius, material)
    }

    /// Adds a high-level object and expands it to primitives.
    pub fn add_object(mut self, object: SceneObject) -> Self {
        let (spheres, triangles) = object.into_primitives();
        self.spheres.extend(spheres);
        self.triangles.extend(triangles);
        self
    }

    /// Adds a triangle primitive.
    pub fn add_triangle(mut self, a: Vec3, b: Vec3, c: Vec3, material: Material) -> Self {
        self.triangles.push(Triangle::flat(a, b, c, material));
        self
    }

    /// Adds a mesh by expanding it to triangles.
    pub fn add_mesh(
        mut self,
        mesh: &crate::core::engine::rendering::mesh::asset::MeshAsset,
        translation: Vec3,
        scale: f64,
        material: Material,
    ) -> Self {
        self.triangles.extend(mesh.to_triangles(translation, scale, material));
        self
    }

    // -----------------------------------------------------------------------
    // Lighting
    // -----------------------------------------------------------------------

    /// Sets the directional light direction.
    pub fn sun_direction(mut self, dir: [f64; 3]) -> Self {
        self.sun_direction = Vec3::new(dir[0], dir[1], dir[2]).normalize();
        self
    }

    /// Sets the directional light color.
    pub fn sun_color(mut self, rgb: [f64; 3]) -> Self {
        self.sun_color = Vec3::new(rgb[0], rgb[1], rgb[2]);
        self
    }

        /// Light position.
        /// Light color.
        /// Light intensity.
        /// Rectangular size.
    /// Sets the directional light intensity.
    pub fn sun_intensity(mut self, intensity: f64) -> Self {
        self.sun_intensity = intensity.max(0.0);
        self
    }

    /// Adds an area light.
    pub fn add_area_light(
        mut self,
        position: [f64; 3],
        color: [f64; 3],
        intensity: f64,
        size: [f64; 2],
    ) -> Self {
        self.area_lights.push(AreaLight {
            position: Vec3::new(position[0], position[1], position[2]),
            u: Vec3::new(size[0], 0.0, 0.0),
            v: Vec3::new(0.0, 0.0, size[1]),
            color: Vec3::new(color[0], color[1], color[2]),
            intensity: intensity.max(0.0),
        });
        self
    }

    // -----------------------------------------------------------------------
    // Environment
        /// Camera descriptor.
        /// Sun direction.
        /// Sun color.
        /// Sun intensity.
        /// Sky top color.
        /// Sky bottom color.
        /// Global exposure.
        /// Sphere entries.
        /// Triangle entries.
        /// Area-light entries.
    // -----------------------------------------------------------------------

    /// Sets top and bottom sky colors.
    pub fn sky(mut self, top: [f64; 3], bottom: [f64; 3]) -> Self {
        self.sky_top = Vec3::new(top[0], top[1], top[2]);
        self.sky_bottom = Vec3::new(bottom[0], bottom[1], bottom[2]);
        self
    }

    /// Sets global scene exposure.
    pub fn exposure(mut self, exposure: f64) -> Self {
        self.exposure = exposure.max(0.01);
        self
    }

    /// Enables a denser volumetric medium preset.
    pub fn with_dense_volume(mut self) -> Self {
        self.volume = VolumetricMedium::cinematic_nebula().with_density_multiplier(1.8);
        self
    }

    /// Uses vacuum medium (no volumetrics).
    pub fn with_vacuum(mut self) -> Self {
        self.volume = VolumetricMedium::vacuum();
        self
    }

    /// Sets an explicit volumetric medium.
    pub fn with_volume(mut self, medium: VolumetricMedium) -> Self {
        self.volume = medium;
        self
    }

    // -----------------------------------------------------------------------
    // Camera
    // -----------------------------------------------------------------------

    /// Replaces camera descriptor.
    pub fn with_camera(mut self, desc: CameraDesc) -> Self {
        self.camera = desc;
        self
    }

    /// Sets camera eye and target positions.
    pub fn camera_position(mut self, eye: [f64; 3], target: [f64; 3]) -> Self {
        self.camera.eye = eye;
        self.camera.target = target;
        self
    }

    /// Sets camera vertical field of view in degrees.
    pub fn camera_fov(mut self, degrees: f64) -> Self {
        self.camera.fov_degrees = degrees.clamp(10.0, 120.0);
        self
    }

    /// Sets camera aperture value.
    pub fn camera_aperture(mut self, aperture: f64) -> Self {
        self.camera.aperture = aperture.max(0.0);
        self
    }

    // -----------------------------------------------------------------------
    // Auto framing
    // -----------------------------------------------------------------------

    /// Automatically frames the current scene content.
    pub fn auto_frame(mut self) -> Self {
        if self.spheres.is_empty() && self.triangles.is_empty() {
            return self;
        }
        let sphere_center_sum = self
            .spheres
            .iter()
            .fold(Vec3::ZERO, |a, s| a + s.center);
        let triangle_center_sum = self
            .triangles
            .iter()
            .fold(Vec3::ZERO, |a, t| a + (t.a + t.b + t.c) / 3.0);
        let sample_count = (self.spheres.len() + self.triangles.len()) as f64;
        let center = (sphere_center_sum + triangle_center_sum) / sample_count.max(1.0);

        let sphere_extent = self
            .spheres
            .iter()
            .map(|s| (s.center - center).length() + s.radius)
            .fold(0.0_f64, f64::max);
        let triangle_extent = self
            .triangles
            .iter()
            .map(|t| {
                (t.a - center)
                    .length()
                    .max((t.b - center).length())
                    .max((t.c - center).length())
            })
            .fold(0.0_f64, f64::max);
        let extent = sphere_extent.max(triangle_extent).max(1.0);
        let dist = extent * 2.8;
        self.camera.eye = [center.x + dist * 0.7, center.y + dist * 0.45, center.z + dist * 0.7];
        self.camera.target = [center.x, center.y, center.z];
        self
    }

    // -----------------------------------------------------------------------
    // Build
    // -----------------------------------------------------------------------

    /// Builds final scene and camera pair for rendering.
    pub fn build(self, aspect_ratio: f64) -> (Scene, Camera) {
        let scene = Scene {
            objects: self.spheres,
            triangles: self.triangles,
            sun: DirectionalLight {
                direction: self.sun_direction,
                color: self.sun_color,
                intensity: self.sun_intensity,
                angular_radius: self.sun_angular_radius,
            },
            area_lights: self.area_lights,
            sky_top: self.sky_top,
            sky_bottom: self.sky_bottom,
            exposure: self.exposure,
            volume: self.volume,
            hdri: None,
            solar_elevation: 0.48,
        };

        let eye = Vec3::new(self.camera.eye[0], self.camera.eye[1], self.camera.eye[2]);
        let target = Vec3::new(
            self.camera.target[0],
            self.camera.target[1],
            self.camera.target[2],
        );
        let mut camera = Camera::look_at(
            eye,
            target,
            Vec3::new(0.0, 1.0, 0.0),
            self.camera.fov_degrees,
            aspect_ratio,
        );
        if self.camera.aperture > 0.0 {
            camera = camera.with_physical_lens(self.camera.aperture, 0.0, Vec3::ZERO);
        }

        (scene, camera)
    }
}

/// Serializable sphere input entry used by descriptor parsing.
#[derive(Debug, Clone)]
pub struct SphereEntry {
    /// Sphere center position.
    pub position: [f64; 3],
    /// Sphere radius.
    pub radius:   f64,
    /// Optional material preset name.
    pub material_name: Option<String>,
    /// Fallback albedo color.
    pub albedo:    [f64; 3],
    /// Surface roughness.
    pub roughness: f64,
    /// Metallic factor.
    pub metallic:  f64,
    /// Emission intensity.
    pub emission:  f64,
}

impl Default for SphereEntry {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            radius: 1.0,
            material_name: None,
            albedo:    [0.8, 0.8, 0.8],
            roughness: 0.5,
            metallic:  0.0,
            emission:  0.0,
        }
    }
}

/// Serializable triangle input entry used by descriptor parsing.
#[derive(Debug, Clone)]
pub struct TriangleEntry {
    /// Vertex A position.
    pub a: [f64; 3],
    /// Vertex B position.
    pub b: [f64; 3],
    /// Vertex C position.
    pub c: [f64; 3],
    /// Optional material preset name.
    pub material_name: Option<String>,
    /// Fallback albedo color.
    pub albedo: [f64; 3],
    /// Surface roughness.
    pub roughness: f64,
    /// Metallic factor.
    pub metallic: f64,
    /// Emission intensity.
    pub emission: f64,
}

impl Default for TriangleEntry {
    fn default() -> Self {
        Self {
            a: [0.0; 3],
            b: [0.0; 3],
            c: [0.0; 3],
            material_name: None,
            albedo: [0.8, 0.8, 0.8],
            roughness: 0.5,
            metallic: 0.0,
            emission: 0.0,
        }
    }
}

#[derive(Debug, Clone)]
/// Serializable area-light entry used by descriptor parsing.
pub struct AreaLightEntry {
    /// Light position.
    pub position:  [f64; 3],
    /// Light color.
    pub color:     [f64; 3],
    /// Light intensity.
    pub intensity: f64,
    /// Rectangular light size.
    pub size:      [f64; 2],
}

impl Default for AreaLightEntry {
    fn default() -> Self {
        Self {
            position:  [0.0, 5.0, 0.0],
            color:     [1.0, 1.0, 1.0],
            intensity: 1.0,
            size:      [2.0, 2.0],
        }
    }
}

#[derive(Debug, Clone)]
/// Serializable scene descriptor format.
pub struct SceneDescriptor {
    /// Camera descriptor.
    pub camera:        CameraDesc,
    /// Sun direction.
    pub sun_direction: [f64; 3],
    /// Sun color.
    pub sun_color:     [f64; 3],
    /// Sun intensity.
    pub sun_intensity: f64,
    /// Sky top color.
    pub sky_top:       [f64; 3],
    /// Sky bottom color.
    pub sky_bottom:    [f64; 3],
    /// Global exposure value.
    pub exposure:      f64,
    /// Sphere entries.
    pub spheres:       Vec<SphereEntry>,
    /// Triangle entries.
    pub triangles:     Vec<TriangleEntry>,
    /// Area-light entries.
    pub area_lights:   Vec<AreaLightEntry>,
}

impl Default for SceneDescriptor {
    fn default() -> Self {
        Self {
            camera:        CameraDesc::default(),
            sun_direction: [-0.65, -0.35, -1.0],
            sun_color:     [1.0, 0.96, 0.90],
            sun_intensity: 1.45,
            sky_top:       [0.015, 0.020, 0.050],
            sky_bottom:    [0.001, 0.001, 0.006],
            exposure:      1.45,
            spheres:       Vec::new(),
            triangles:     Vec::new(),
            area_lights:   Vec::new(),
        }
    }
}

impl SceneDescriptor {
    /// Loads a descriptor from a `.scene` text queue.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let path_ref = path.as_ref();
        let metadata = fs::metadata(path_ref)?;
        let size = metadata.len();
        if size > MAX_SCENE_FILE_SIZE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("scene file size {size} exceeds limit {MAX_SCENE_FILE_SIZE}"),
            ));
        }
        let text = fs::read_to_string(path_ref)?;
        Self::parse(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Saves the descriptor to a `.scene` text queue.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if let Some(parent) = path.as_ref().parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.serialize())
    }

    /// Parses a descriptor from raw text content.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut desc = SceneDescriptor::default();

        for (zero_based, raw_line) in text.lines().enumerate() {
            let line_number = zero_based + 1;
            let stripped = if zero_based == 0 {
                raw_line.strip_prefix('\u{feff}').unwrap_or(raw_line)
            } else {
                raw_line
            };
            let line = stripped.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let (keyword, rest) = line
                .split_once(' ')
                .map(|(k, r)| (k, r.trim()))
                .unwrap_or((line, ""));

            match keyword {
                "version" => {}
                "camera" => {
                    let kv = KvMap::parse(rest, line_number)?;
                    if let Some(v) = kv.get("eye")      { desc.camera.eye         = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("target")   { desc.camera.target      = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("fov")      {
                        let value = parse_f64_val(v, line_number)?;
                        if value <= 0.0 || value >= 180.0 {
                            return Err(format!("line {line_number}: fov must be in (0, 180), got {value}"));
                        }
                        desc.camera.fov_degrees = value;
                    }
                    if let Some(v) = kv.get("aperture") {
                        let value = parse_f64_val(v, line_number)?;
                        if value < 0.0 {
                            return Err(format!("line {line_number}: aperture must be non-negative, got {value}"));
                        }
                        desc.camera.aperture = value;
                    }
                }
                "sun" => {
                    let kv = KvMap::parse(rest, line_number)?;
                    if let Some(v) = kv.get("dir")       { desc.sun_direction = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("intensity") {
                        let value = parse_f64_val(v, line_number)?;
                        if value < 0.0 {
                            return Err(format!("line {line_number}: sun intensity must be non-negative, got {value}"));
                        }
                        desc.sun_intensity = value;
                    }
                    if let Some(v) = kv.get("color")     { desc.sun_color     = parse_f64_vec3(v, line_number)?; }
                }
                "sky" => {
                    let kv = KvMap::parse(rest, line_number)?;
                    if let Some(v) = kv.get("top")    { desc.sky_top    = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("bottom") { desc.sky_bottom = parse_f64_vec3(v, line_number)?; }
                }
                "exposure" => {
                    let value = parse_f64_val(rest, line_number)?;
                    if value <= 0.0 {
                        return Err(format!("line {line_number}: exposure must be positive, got {value}"));
                    }
                    desc.exposure = value;
                }
                "sphere" => {
                    if desc.spheres.len() >= MAX_SCENE_SPHERES {
                        return Err(format!(
                            "line {line_number}: sphere count exceeds limit {MAX_SCENE_SPHERES}"
                        ));
                    }
                    let kv = KvMap::parse(rest, line_number)?;
                    let mut entry = SphereEntry::default();
                    if let Some(v) = kv.get("pos")       { entry.position      = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("radius")    {
                        let value = parse_f64_val(v, line_number)?;
                        if value <= 0.0 {
                            return Err(format!("line {line_number}: sphere radius must be positive, got {value}"));
                        }
                        entry.radius = value;
                    }
                    if let Some(v) = kv.get("material")  {
                        validate_material_name(v, line_number)?;
                        entry.material_name = Some(v.to_string());
                    }
                    if let Some(v) = kv.get("albedo")    { entry.albedo        = parse_unit_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("roughness") { entry.roughness     = parse_unit_scalar(v, line_number)?; }
                    if let Some(v) = kv.get("metallic")  { entry.metallic      = parse_unit_scalar(v, line_number)?; }
                    if let Some(v) = kv.get("emission")  {
                        let value = parse_f64_val(v, line_number)?;
                        if value < 0.0 {
                            return Err(format!("line {line_number}: emission must be non-negative, got {value}"));
                        }
                        entry.emission = value;
                    }
                    desc.spheres.push(entry);
                }
                "triangle" => {
                    if desc.triangles.len() >= MAX_SCENE_TRIANGLES {
                        return Err(format!(
                            "line {line_number}: triangle count exceeds limit {MAX_SCENE_TRIANGLES}"
                        ));
                    }
                    let kv = KvMap::parse(rest, line_number)?;
                    let mut entry = TriangleEntry::default();
                    if let Some(v) = kv.get("a")         { entry.a             = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("b")         { entry.b             = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("c")         { entry.c             = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("material")  {
                        validate_material_name(v, line_number)?;
                        entry.material_name = Some(v.to_string());
                    }
                    if let Some(v) = kv.get("albedo")    { entry.albedo        = parse_unit_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("roughness") { entry.roughness     = parse_unit_scalar(v, line_number)?; }
                    if let Some(v) = kv.get("metallic")  { entry.metallic      = parse_unit_scalar(v, line_number)?; }
                    if let Some(v) = kv.get("emission")  {
                        let value = parse_f64_val(v, line_number)?;
                        if value < 0.0 {
                            return Err(format!("line {line_number}: emission must be non-negative, got {value}"));
                        }
                        entry.emission = value;
                    }
                    desc.triangles.push(entry);
                }
                "area_light" => {
                    if desc.area_lights.len() >= MAX_SCENE_AREA_LIGHTS {
                        return Err(format!(
                            "line {line_number}: area_light count exceeds limit {MAX_SCENE_AREA_LIGHTS}"
                        ));
                    }
                    let kv = KvMap::parse(rest, line_number)?;
                    let mut entry = AreaLightEntry::default();
                    if let Some(v) = kv.get("pos")       { entry.position  = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("color")     { entry.color     = parse_f64_vec3(v, line_number)?; }
                    if let Some(v) = kv.get("intensity") {
                        let value = parse_f64_val(v, line_number)?;
                        if value < 0.0 {
                            return Err(format!("line {line_number}: area_light intensity must be non-negative, got {value}"));
                        }
                        entry.intensity = value;
                    }
                    if let Some(v) = kv.get("size")      {
                        let size = parse_f64_vec2(v, line_number)?;
                        if size[0] <= 0.0 || size[1] <= 0.0 {
                            return Err(format!("line {line_number}: area_light size must be positive, got {},{}", size[0], size[1]));
                        }
                        entry.size = size;
                    }
                    desc.area_lights.push(entry);
                }
                other => {
                    return Err(format!("line {line_number}: unknown keyword '{other}'"));
                }
            }
        }

        Ok(desc)
    }

    /// Serializes the descriptor to text format.
    pub fn serialize(&self) -> String {
        let mut s = String::new();
        s.push_str("version 1\n");

        s.push_str(&format!(
            "camera eye={} target={} fov={:.4} aperture={:.6}\n",
            fmt3(self.camera.eye),
            fmt3(self.camera.target),
            self.camera.fov_degrees,
            self.camera.aperture,
        ));
        s.push_str(&format!(
            "sun dir={} intensity={:.4} color={}\n",
            fmt3(self.sun_direction),
            self.sun_intensity,
            fmt3(self.sun_color),
        ));
        s.push_str(&format!(
            "sky top={} bottom={}\n",
            fmt3(self.sky_top),
            fmt3(self.sky_bottom),
        ));
        s.push_str(&format!("exposure {:.4}\n", self.exposure));

        for sphere in &self.spheres {
            if let Some(mat) = &sphere.material_name {
                s.push_str(&format!(
                    "sphere pos={} radius={:.4} material={}\n",
                    fmt3(sphere.position),
                    sphere.radius,
                    mat,
                ));
            } else {
                s.push_str(&format!(
                    "sphere pos={} radius={:.4} albedo={} roughness={:.4} metallic={:.4} emission={:.4}\n",
                    fmt3(sphere.position),
                    sphere.radius,
                    fmt3(sphere.albedo),
                    sphere.roughness,
                    sphere.metallic,
                    sphere.emission,
                ));
            }
        }

        for triangle in &self.triangles {
            if let Some(mat) = &triangle.material_name {
                s.push_str(&format!(
                    "triangle a={} b={} c={} material={}\n",
                    fmt3(triangle.a),
                    fmt3(triangle.b),
                    fmt3(triangle.c),
                    mat,
                ));
            } else {
                s.push_str(&format!(
                    "triangle a={} b={} c={} albedo={} roughness={:.4} metallic={:.4} emission={:.4}\n",
                    fmt3(triangle.a),
                    fmt3(triangle.b),
                    fmt3(triangle.c),
                    fmt3(triangle.albedo),
                    triangle.roughness,
                    triangle.metallic,
                    triangle.emission,
                ));
            }
        }

        for light in &self.area_lights {
            s.push_str(&format!(
                "area_light pos={} color={} intensity={:.4} size={}\n",
                fmt3(light.position),
                fmt3(light.color),
                light.intensity,
                fmt2(light.size),
            ));
        }

        s
    }

    /// Converts this descriptor into a fluent `SceneBuilder`.
    pub fn into_builder(self) -> SceneBuilder {
        let mut builder = SceneBuilder::new()
            .sun_direction(self.sun_direction)
            .sun_color(self.sun_color)
            .sun_intensity(self.sun_intensity)
            .sky(self.sky_top, self.sky_bottom)
            .exposure(self.exposure)
            .with_camera(self.camera);

        for entry in self.spheres {
            let material = if let Some(name) = &entry.material_name {
                MaterialCatalog.by_name(name)
            } else {
                Material::new(
                    Vec3::new(entry.albedo[0], entry.albedo[1], entry.albedo[2]),
                    entry.roughness,
                    entry.metallic,
                    entry.metallic.clamp(0.0, 1.0),
                    Vec3::new(entry.emission, entry.emission, entry.emission),
                )
            };
            builder = builder.add_sphere(
                Vec3::new(entry.position[0], entry.position[1], entry.position[2]),
                entry.radius,
                material,
            );
        }

        for entry in self.triangles {
            let material = if let Some(name) = &entry.material_name {
                MaterialCatalog.by_name(name)
            } else {
                Material::new(
                    Vec3::new(entry.albedo[0], entry.albedo[1], entry.albedo[2]),
                    entry.roughness,
                    entry.metallic,
                    entry.metallic.clamp(0.0, 1.0),
                    Vec3::new(entry.emission, entry.emission, entry.emission),
                )
            };
            builder = builder.add_triangle(
                Vec3::new(entry.a[0], entry.a[1], entry.a[2]),
                Vec3::new(entry.b[0], entry.b[1], entry.b[2]),
                Vec3::new(entry.c[0], entry.c[1], entry.c[2]),
                material,
            );
        }

        for light in self.area_lights {
            builder = builder.add_area_light(light.position, light.color, light.intensity, light.size);
        }

        builder
    }
}

struct KvMap<'a> {
    pairs: Vec<(&'a str, &'a str)>,
}

impl<'a> KvMap<'a> {
    fn parse(s: &'a str, line_number: usize) -> Result<Self, String> {
        let mut pairs = Vec::new();
        for token in s.split_whitespace() {
            let (key, value) = token.split_once('=').ok_or_else(|| {
                format!("line {line_number}: malformed token '{token}', expected key=value")
            })?;
            if key.is_empty() {
                return Err(format!("line {line_number}: empty key in token '{token}'"));
            }
            if value.is_empty() {
                return Err(format!("line {line_number}: empty value for key '{key}'"));
            }
            pairs.push((key, value));
        }
        Ok(Self { pairs })
    }

    fn get(&self, key: &str) -> Option<&'a str> {
        self.pairs.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}

/// Maximum size for a `.scene` file (8 MiB).
pub const MAX_SCENE_FILE_SIZE: u64 = 8 * 1024 * 1024;
/// Maximum number of spheres per scene.
pub const MAX_SCENE_SPHERES: usize = 100_000;
/// Maximum number of triangles per scene.
pub const MAX_SCENE_TRIANGLES: usize = 1_000_000;
/// Maximum number of area lights per scene.
pub const MAX_SCENE_AREA_LIGHTS: usize = 1_024;
/// Maximum length of a material name.
pub const MAX_MATERIAL_NAME_LEN: usize = 128;

fn parse_f64_val(s: &str, line_number: usize) -> Result<f64, String> {
    let trimmed = s.trim();
    let value = f64::from_str(trimmed)
        .map_err(|_| format!("line {line_number}: invalid f64 '{trimmed}'"))?;
    if !value.is_finite() {
        return Err(format!("line {line_number}: non-finite f64 '{trimmed}'"));
    }
    Ok(value)
}

fn parse_unit_scalar(s: &str, line_number: usize) -> Result<f64, String> {
    let value = parse_f64_val(s, line_number)?;
    if !(0.0..=1.0).contains(&value) {
        return Err(format!(
            "line {line_number}: scalar must be within [0, 1], got {value}"
        ));
    }
    Ok(value)
}

fn parse_f64_vec3(s: &str, line_number: usize) -> Result<[f64; 3], String> {
    let parts: Vec<&str> = s.splitn(3, ',').collect();
    if parts.len() != 3 {
        return Err(format!(
            "line {line_number}: expected 3 components in '{s}'"
        ));
    }
    Ok([
        parse_f64_val(parts[0], line_number)?,
        parse_f64_val(parts[1], line_number)?,
        parse_f64_val(parts[2], line_number)?,
    ])
}

fn parse_unit_vec3(s: &str, line_number: usize) -> Result<[f64; 3], String> {
    let raw = parse_f64_vec3(s, line_number)?;
    for component in raw {
        if !(0.0..=1.0).contains(&component) {
            return Err(format!(
                "line {line_number}: color components must be within [0, 1], got {component}"
            ));
        }
    }
    Ok(raw)
}

fn parse_f64_vec2(s: &str, line_number: usize) -> Result<[f64; 2], String> {
    let parts: Vec<&str> = s.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(format!(
            "line {line_number}: expected 2 components in '{s}'"
        ));
    }
    Ok([
        parse_f64_val(parts[0], line_number)?,
        parse_f64_val(parts[1], line_number)?,
    ])
}

fn validate_material_name(name: &str, line_number: usize) -> Result<(), String> {
    if name.is_empty() {
        return Err(format!("line {line_number}: empty material name"));
    }
    if name.len() > MAX_MATERIAL_NAME_LEN {
        return Err(format!(
            "line {line_number}: material name length {} exceeds limit {MAX_MATERIAL_NAME_LEN}",
            name.len()
        ));
    }
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(format!(
            "line {line_number}: material name '{name}' contains invalid characters"
        ));
    }
    Ok(())
}

fn fmt3(v: [f64; 3]) -> String {
    format!("{},{},{}", v[0], v[1], v[2])
}

fn fmt2(v: [f64; 2]) -> String {
    format!("{},{}", v[0], v[1])
}

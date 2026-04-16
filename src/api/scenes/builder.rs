use std::{fs, io, path::Path, str::FromStr};

use crate::api::materials::catalog::MaterialCatalog;
use crate::api::objects::SceneObject;
use crate::api::types::CameraDesc;
use crate::core::engine::rendering::raytracing::{
    AreaLight, Camera, DirectionalLight, Material, Scene, Sphere, Triangle, Vec3,
};
use crate::core::engine::rendering::effects::volumetric_effects::medium::VolumetricMedium;

/// Fluent, AI-friendly scene builder.
///
/// ```ignore
/// let scene = SceneBuilder::new()
///     .add_sphere(Vec3::new(0.0, 0.0, 0.0), 1.6, mat_star)
///     .add_sphere(Vec3::new(5.0, 0.3, 0.0), 0.55, mat_planet)
///     .sun_direction([-0.6, -0.4, -1.0])
///     .sun_intensity(1.5)
///     .with_camera(CameraDesc { eye: [12.0, 6.0, 12.0], ..Default::default() })
///     .build(16.0 / 9.0);
/// ```
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
    /// Crée un `SceneBuilder` avec environnement et lumière par défaut.
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

    /// Ajoute une sphère explicite.
    pub fn add_sphere(mut self, center: Vec3, radius: f64, material: Material) -> Self {
        self.spheres.push(Sphere {
            center,
            radius: radius.max(0.01),
            material,
        });
        self
    }

    /// Ajoute une sphère en résolvant le matériau par son nom.
    pub fn add_sphere_named(self, center: Vec3, radius: f64, material_name: &str) -> Self {
        let material = MaterialCatalog.by_name(material_name);
        self.add_sphere(center, radius, material)
    }

    /// Add a [`SceneObject`] (sphere, triangle, or composite group).
    /// Groups are flattened recursively.
    pub fn add_object(mut self, object: SceneObject) -> Self {
        let (spheres, triangles) = object.into_primitives();
        self.spheres.extend(spheres);
        self.triangles.extend(triangles);
        self
    }

    /// Add a raw triangle.
    pub fn add_triangle(mut self, a: Vec3, b: Vec3, c: Vec3, material: Material) -> Self {
        self.triangles.push(Triangle::flat(a, b, c, material));
        self
    }

    /// Add all triangles from a [`MeshAsset`] with the given transform and material.
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

    /// Définit la direction solaire.
    pub fn sun_direction(mut self, dir: [f64; 3]) -> Self {
        self.sun_direction = Vec3::new(dir[0], dir[1], dir[2]).normalize();
        self
    }

    /// Définit la couleur solaire.
    pub fn sun_color(mut self, rgb: [f64; 3]) -> Self {
        self.sun_color = Vec3::new(rgb[0], rgb[1], rgb[2]);
        self
    }

    /// Définit l'intensité solaire.
    pub fn sun_intensity(mut self, intensity: f64) -> Self {
        self.sun_intensity = intensity.max(0.0);
        self
    }

    /// Ajoute une lumière de surface rectangulaire.
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
    // -----------------------------------------------------------------------

    /// Définit les couleurs de ciel haut/bas.
    pub fn sky(mut self, top: [f64; 3], bottom: [f64; 3]) -> Self {
        self.sky_top = Vec3::new(top[0], top[1], top[2]);
        self.sky_bottom = Vec3::new(bottom[0], bottom[1], bottom[2]);
        self
    }

    /// Définit l'exposition globale.
    pub fn exposure(mut self, exposure: f64) -> Self {
        self.exposure = exposure.max(0.01);
        self
    }

    /// Active un volume dense prédéfini.
    pub fn with_dense_volume(mut self) -> Self {
        self.volume = VolumetricMedium::cinematic_nebula().with_density_multiplier(1.8);
        self
    }

    /// Active un volume de vide.
    pub fn with_vacuum(mut self) -> Self {
        self.volume = VolumetricMedium::vacuum();
        self
    }

    /// Set a custom volumetric medium.
    pub fn with_volume(mut self, medium: VolumetricMedium) -> Self {
        self.volume = medium;
        self
    }

    // -----------------------------------------------------------------------
    // Camera
    // -----------------------------------------------------------------------

    /// Définit la caméra complète.
    pub fn with_camera(mut self, desc: CameraDesc) -> Self {
        self.camera = desc;
        self
    }

    /// Définit position et cible caméra.
    pub fn camera_position(mut self, eye: [f64; 3], target: [f64; 3]) -> Self {
        self.camera.eye = eye;
        self.camera.target = target;
        self
    }

    /// Définit le champ de vision vertical.
    pub fn camera_fov(mut self, degrees: f64) -> Self {
        self.camera.fov_degrees = degrees.clamp(10.0, 120.0);
        self
    }

    /// Définit l'ouverture caméra.
    pub fn camera_aperture(mut self, aperture: f64) -> Self {
        self.camera.aperture = aperture.max(0.0);
        self
    }

    // -----------------------------------------------------------------------
    // Auto framing
    // -----------------------------------------------------------------------

    /// Automatically position the camera to frame all objects.
    pub fn auto_frame(mut self) -> Self {
        if self.spheres.is_empty() {
            return self;
        }
        let center = self
            .spheres
            .iter()
            .fold(Vec3::ZERO, |a, s| a + s.center)
            / self.spheres.len() as f64;
        let extent = self
            .spheres
            .iter()
            .map(|s| (s.center - center).length() + s.radius)
            .fold(1.0_f64, f64::max);
        let dist = extent * 2.8;
        self.camera.eye = [center.x + dist * 0.7, center.y + dist * 0.45, center.z + dist * 0.7];
        self.camera.target = [center.x, center.y, center.z];
        self
    }

    // -----------------------------------------------------------------------
    // Build
    // -----------------------------------------------------------------------

    /// Consume the builder and produce a renderable `(Scene, Camera)`.
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

/// Données brutes d'une sphère dans un descripteur de scène.
#[derive(Debug, Clone)]
pub struct SphereEntry {
    /// Position dans l'espace monde `[x, y, z]`.
    pub position: [f64; 3],
    /// Rayon de la sphère, en unités monde.
    pub radius:   f64,
    /// Nom du matériau du catalogue, prioritaire sur les champs PBR ci-dessous.
    pub material_name: Option<String>,
    /// Couleur de base (albédo) `[r, g, b]` en espace linéaire.
    pub albedo:    [f64; 3],
    /// Rugosité de la surface, de `0.0` (miroir) à `1.0` (totalement mat).
    pub roughness: f64,
    /// Facteur métallique de la surface, de `0.0` (diélectrique) à `1.0` (métal).
    pub metallic:  f64,
    /// Intensité d'émission de la surface.
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

/// Données brutes d'un triangle dans un descripteur de scène.
#[derive(Debug, Clone)]
pub struct TriangleEntry {
    /// Premier sommet du triangle `[x, y, z]`.
    pub a: [f64; 3],
    /// Deuxième sommet du triangle `[x, y, z]`.
    pub b: [f64; 3],
    /// Troisième sommet du triangle `[x, y, z]`.
    pub c: [f64; 3],
    /// Nom du matériau du catalogue, prioritaire sur les champs PBR ci-dessous.
    pub material_name: Option<String>,
    /// Couleur de base (albédo) `[r, g, b]` en espace linéaire.
    pub albedo: [f64; 3],
    /// Rugosité de la surface, de `0.0` (miroir) à `1.0` (totalement mat).
    pub roughness: f64,
    /// Facteur métallique de la surface, de `0.0` (diélectrique) à `1.0` (métal).
    pub metallic: f64,
    /// Intensité d'émission de la surface.
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

/// Données brutes d'une lumière surfacique dans un descripteur de scène.
#[derive(Debug, Clone)]
pub struct AreaLightEntry {
    /// Position du centre de la lumière `[x, y, z]`.
    pub position:  [f64; 3],
    /// Couleur émise `[r, g, b]` en espace linéaire.
    pub color:     [f64; 3],
    /// Intensité lumineuse.
    pub intensity: f64,
    /// Dimensions de la surface `[largeur, hauteur]`.
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

/// Descripteur déclaratif d'une scène complète, sérialisable vers/depuis un fichier texte.
///
/// Utilisé pour décrire la scène de façon portable avant de la convertir en [`SceneBuilder`]
/// exécutable via [`SceneDescriptor::into_builder`].
#[derive(Debug, Clone)]
pub struct SceneDescriptor {
    /// Description de la caméra (position, cible, champ de vision, ouverture).
    pub camera:        CameraDesc,
    /// Direction du soleil `[x, y, z]` (non normalisée, normalisée à la construction).
    pub sun_direction: [f64; 3],
    /// Couleur du soleil `[r, g, b]` en espace linéaire.
    pub sun_color:     [f64; 3],
    /// Intensité du soleil.
    pub sun_intensity: f64,
    /// Couleur du ciel en haut `[r, g, b]`.
    pub sky_top:       [f64; 3],
    /// Couleur du ciel en bas (horizon) `[r, g, b]`.
    pub sky_bottom:    [f64; 3],
    /// Facteur d'exposition global de la scène.
    pub exposure:      f64,
    /// Liste des sphères de la scène.
    pub spheres:       Vec<SphereEntry>,
    /// Liste des triangles de la scène.
    pub triangles:     Vec<TriangleEntry>,
    /// Liste des lumières surfaciques de la scène.
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
    /// Charge un descripteur de scène depuis un fichier texte.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        Self::parse(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    /// Sérialise et enregistre le descripteur dans un fichier texte.
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if let Some(parent) = path.as_ref().parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.serialize())
    }

    /// Parse un descripteur de scène depuis une chaîne de texte.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut desc = SceneDescriptor::default();

        for raw_line in text.lines() {
            let line = raw_line.trim();
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
                    let kv = KvMap::parse(rest);
                    if let Some(v) = kv.get("eye")   { desc.camera.eye    = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("target") { desc.camera.target = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("fov")    { desc.camera.fov_degrees = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("aperture") { desc.camera.aperture = parse_f64_val(v)?; }
                }
                "sun" => {
                    let kv = KvMap::parse(rest);
                    if let Some(v) = kv.get("dir")       { desc.sun_direction = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("intensity")  { desc.sun_intensity  = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("color")      { desc.sun_color      = parse_f64_vec3(v)?; }
                }
                "sky" => {
                    let kv = KvMap::parse(rest);
                    if let Some(v) = kv.get("top")    { desc.sky_top    = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("bottom") { desc.sky_bottom = parse_f64_vec3(v)?; }
                }
                "exposure" => {
                    desc.exposure = parse_f64_val(rest)?;
                }
                "sphere" => {
                    let kv = KvMap::parse(rest);
                    let mut entry = SphereEntry::default();
                    if let Some(v) = kv.get("pos")       { entry.position      = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("radius")    { entry.radius        = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("material")  { entry.material_name = Some(v.to_string()); }
                    if let Some(v) = kv.get("albedo")    { entry.albedo        = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("roughness") { entry.roughness     = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("metallic")  { entry.metallic      = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("emission")  { entry.emission      = parse_f64_val(v)?; }
                    desc.spheres.push(entry);
                }
                "triangle" => {
                    let kv = KvMap::parse(rest);
                    let mut entry = TriangleEntry::default();
                    if let Some(v) = kv.get("a")         { entry.a             = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("b")         { entry.b             = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("c")         { entry.c             = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("material")  { entry.material_name = Some(v.to_string()); }
                    if let Some(v) = kv.get("albedo")    { entry.albedo        = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("roughness") { entry.roughness     = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("metallic")  { entry.metallic      = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("emission")  { entry.emission      = parse_f64_val(v)?; }
                    desc.triangles.push(entry);
                }
                "area_light" => {
                    let kv = KvMap::parse(rest);
                    let mut entry = AreaLightEntry::default();
                    if let Some(v) = kv.get("pos")       { entry.position  = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("color")     { entry.color     = parse_f64_vec3(v)?; }
                    if let Some(v) = kv.get("intensity") { entry.intensity = parse_f64_val(v)?; }
                    if let Some(v) = kv.get("size")      { entry.size      = parse_f64_vec2(v)?; }
                    desc.area_lights.push(entry);
                }
                other => {
                    return Err(format!("unknown keyword '{other}'"));
                }
            }
        }

        Ok(desc)
    }

    /// Sérialise le descripteur en chaîne de texte.
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

    /// Convertit le descripteur en [`SceneBuilder`] prêt à être rendu.
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
                    entry.metallic,
                    Vec3::new(entry.emission, entry.emission, entry.emission) * entry.emission,
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
                    entry.metallic,
                    Vec3::new(entry.emission, entry.emission, entry.emission) * entry.emission,
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
    fn parse(s: &'a str) -> Self {
        let pairs = s.split_whitespace()
            .filter_map(|token| token.split_once('='))
            .collect();
        Self { pairs }
    }

    fn get(&self, key: &str) -> Option<&'a str> {
        self.pairs.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
    }
}

fn parse_f64_val(s: &str) -> Result<f64, String> {
    f64::from_str(s.trim()).map_err(|_| format!("invalid f64 '{s}'"))
}

fn parse_f64_vec3(s: &str) -> Result<[f64; 3], String> {
    let parts: Vec<&str> = s.splitn(3, ',').collect();
    if parts.len() != 3 {
        return Err(format!("expected 3 components in '{s}'"));
    }
    Ok([parse_f64_val(parts[0])?, parse_f64_val(parts[1])?, parse_f64_val(parts[2])?])
}

fn parse_f64_vec2(s: &str) -> Result<[f64; 2], String> {
    let parts: Vec<&str> = s.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(format!("expected 2 components in '{s}'"));
    }
    Ok([parse_f64_val(parts[0])?, parse_f64_val(parts[1])?])
}

fn fmt3(v: [f64; 3]) -> String {
    format!("{},{},{}", v[0], v[1], v[2])
}

fn fmt2(v: [f64; 2]) -> String {
    format!("{},{}", v[0], v[1])
}

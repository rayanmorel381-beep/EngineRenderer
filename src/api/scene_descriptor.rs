use std::{
    fs,
    io,
    path::Path,
    str::FromStr,
};

use crate::api::{
    materials::catalog::MaterialCatalog,
    scenes::builder::SceneBuilder,
    types::CameraDesc,
};
use crate::core::engine::rendering::raytracing::{Material, Vec3};

#[derive(Debug, Clone)]
pub struct SphereEntry {
    pub position: [f64; 3],
    pub radius:   f64,
    pub material_name: Option<String>,
    pub albedo:    [f64; 3],
    pub roughness: f64,
    pub metallic:  f64,
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

#[derive(Debug, Clone)]
pub struct AreaLightEntry {
    pub position:  [f64; 3],
    pub color:     [f64; 3],
    pub intensity: f64,
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
pub struct SceneDescriptor {
    pub camera:        CameraDesc,
    pub sun_direction: [f64; 3],
    pub sun_color:     [f64; 3],
    pub sun_intensity: f64,
    pub sky_top:       [f64; 3],
    pub sky_bottom:    [f64; 3],
    pub exposure:      f64,
    pub spheres:       Vec<SphereEntry>,
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
            area_lights:   Vec::new(),
        }
    }
}

impl SceneDescriptor {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        Self::parse(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        if let Some(parent) = path.as_ref().parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, self.serialize())
    }

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
                    if let Some(v) = kv.get("eye")   { desc.camera.eye    = parse_vec3(v)?; }
                    if let Some(v) = kv.get("target") { desc.camera.target = parse_vec3(v)?; }
                    if let Some(v) = kv.get("fov")    { desc.camera.fov_degrees = parse_f64(v)?; }
                    if let Some(v) = kv.get("aperture") { desc.camera.aperture = parse_f64(v)?; }
                }
                "sun" => {
                    let kv = KvMap::parse(rest);
                    if let Some(v) = kv.get("dir")       { desc.sun_direction = parse_vec3(v)?; }
                    if let Some(v) = kv.get("intensity")  { desc.sun_intensity  = parse_f64(v)?; }
                    if let Some(v) = kv.get("color")      { desc.sun_color      = parse_vec3(v)?; }
                }
                "sky" => {
                    let kv = KvMap::parse(rest);
                    if let Some(v) = kv.get("top")    { desc.sky_top    = parse_vec3(v)?; }
                    if let Some(v) = kv.get("bottom") { desc.sky_bottom = parse_vec3(v)?; }
                }
                "exposure" => {
                    desc.exposure = parse_f64(rest)?;
                }
                "sphere" => {
                    let kv = KvMap::parse(rest);
                    let mut entry = SphereEntry::default();
                    if let Some(v) = kv.get("pos")       { entry.position      = parse_vec3(v)?; }
                    if let Some(v) = kv.get("radius")    { entry.radius        = parse_f64(v)?; }
                    if let Some(v) = kv.get("material")  { entry.material_name = Some(v.to_string()); }
                    if let Some(v) = kv.get("albedo")    { entry.albedo        = parse_vec3(v)?; }
                    if let Some(v) = kv.get("roughness") { entry.roughness     = parse_f64(v)?; }
                    if let Some(v) = kv.get("metallic")  { entry.metallic      = parse_f64(v)?; }
                    if let Some(v) = kv.get("emission")  { entry.emission      = parse_f64(v)?; }
                    desc.spheres.push(entry);
                }
                "area_light" => {
                    let kv = KvMap::parse(rest);
                    let mut entry = AreaLightEntry::default();
                    if let Some(v) = kv.get("pos")       { entry.position  = parse_vec3(v)?; }
                    if let Some(v) = kv.get("color")     { entry.color     = parse_vec3(v)?; }
                    if let Some(v) = kv.get("intensity") { entry.intensity = parse_f64(v)?; }
                    if let Some(v) = kv.get("size")      { entry.size      = parse_vec2(v)?; }
                    desc.area_lights.push(entry);
                }
                other => {
                    return Err(format!("unknown keyword '{other}'"));
                }
            }
        }

        Ok(desc)
    }

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

fn parse_f64(s: &str) -> Result<f64, String> {
    f64::from_str(s.trim()).map_err(|_| format!("invalid f64 '{s}'"))
}

fn parse_vec3(s: &str) -> Result<[f64; 3], String> {
    let parts: Vec<&str> = s.splitn(3, ',').collect();
    if parts.len() != 3 {
        return Err(format!("expected 3 components in '{s}'"));
    }
    Ok([parse_f64(parts[0])?, parse_f64(parts[1])?, parse_f64(parts[2])?])
}

fn parse_vec2(s: &str) -> Result<[f64; 2], String> {
    let parts: Vec<&str> = s.splitn(2, ',').collect();
    if parts.len() != 2 {
        return Err(format!("expected 2 components in '{s}'"));
    }
    Ok([parse_f64(parts[0])?, parse_f64(parts[1])?])
}

fn fmt3(v: [f64; 3]) -> String {
    format!("{},{},{}", v[0], v[1], v[2])
}

fn fmt2(v: [f64; 2]) -> String {
    format!("{},{}", v[0], v[1])
}

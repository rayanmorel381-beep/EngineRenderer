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

    pub fn add_sphere(mut self, center: Vec3, radius: f64, material: Material) -> Self {
        self.spheres.push(Sphere {
            center,
            radius: radius.max(0.01),
            material,
        });
        self
    }

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

    pub fn sun_direction(mut self, dir: [f64; 3]) -> Self {
        self.sun_direction = Vec3::new(dir[0], dir[1], dir[2]).normalize();
        self
    }

    pub fn sun_color(mut self, rgb: [f64; 3]) -> Self {
        self.sun_color = Vec3::new(rgb[0], rgb[1], rgb[2]);
        self
    }

    pub fn sun_intensity(mut self, intensity: f64) -> Self {
        self.sun_intensity = intensity.max(0.0);
        self
    }

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

    pub fn sky(mut self, top: [f64; 3], bottom: [f64; 3]) -> Self {
        self.sky_top = Vec3::new(top[0], top[1], top[2]);
        self.sky_bottom = Vec3::new(bottom[0], bottom[1], bottom[2]);
        self
    }

    pub fn exposure(mut self, exposure: f64) -> Self {
        self.exposure = exposure.max(0.01);
        self
    }

    pub fn with_dense_volume(mut self) -> Self {
        self.volume = VolumetricMedium::cinematic_nebula().with_density_multiplier(1.8);
        self
    }

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

    pub fn with_camera(mut self, desc: CameraDesc) -> Self {
        self.camera = desc;
        self
    }

    pub fn camera_position(mut self, eye: [f64; 3], target: [f64; 3]) -> Self {
        self.camera.eye = eye;
        self.camera.target = target;
        self
    }

    pub fn camera_fov(mut self, degrees: f64) -> Self {
        self.camera.fov_degrees = degrees.clamp(10.0, 120.0);
        self
    }

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

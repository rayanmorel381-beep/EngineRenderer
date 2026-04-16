use super::math::Vec3;
use crate::core::engine::rendering::{
    lod::selection::LodSelection,
    texture::procedural_texture::ProceduralTexture,
};

pub const EPSILON: f64 = 0.001;

/// Rayon avec direction et inverse précalculée.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Origine du rayon.
    pub origin: Vec3,
    /// Direction normalisée ou non.
    pub direction: Vec3,
    /// Inverse composante par composante de la direction.
    pub inv_direction: Vec3,
}

impl Ray {
    /// Construit un rayon et pré-calcule `inv_direction`.
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction,
            inv_direction: Vec3::new(
                1.0 / direction.x,
                1.0 / direction.y,
                1.0 / direction.z,
            ),
        }
    }

    /// Retourne le point du rayon à la distance `distance`.
    pub fn at(self, distance: f64) -> Vec3 {
        self.origin + self.direction * distance
    }
}

/// Matériau PBR étendu utilisé par le ray tracer.
#[derive(Debug, Clone, Copy)]
pub struct Material {
    /// Couleur de base.
    pub albedo: Vec3,
    /// Rugosité micro-facettes.
    pub roughness: f64,
    /// Métallicité.
    pub metallic: f64,
    /// Réflectivité globale.
    pub reflectivity: f64,
    /// Émission lumineuse.
    pub emission: Vec3,
    /// Occlusion ambiante.
    pub ambient_occlusion: f64,
    /// Couche clearcoat.
    pub clearcoat: f64,
    /// Transmission.
    pub transmission: f64,
    /// Indice de réfraction.
    pub ior: f64,
    /// Sheen coloré.
    pub sheen: Vec3,
    /// Subsurface scattering simplifié.
    pub subsurface: f64,
    /// Anisotropie.
    pub anisotropy: f64,
    /// Iridescence.
    pub iridescence: f64,
    /// Poids de texture procédurale.
    pub texture_weight: f64,
    /// Intensité de normal map.
    pub normal_map_strength: f64,
    /// Échelle UV.
    pub uv_scale: f64,
}

impl Material {
    /// Construit un matériau de base.
    pub const fn new(
        albedo: Vec3,
        roughness: f64,
        metallic: f64,
        reflectivity: f64,
        emission: Vec3,
    ) -> Self {
        Self {
            albedo,
            roughness,
            metallic,
            reflectivity,
            emission,
            ambient_occlusion: 1.0,
            clearcoat: 0.0,
            transmission: 0.0,
            ior: 1.45,
            sheen: Vec3::ZERO,
            subsurface: 0.0,
            anisotropy: 0.0,
            iridescence: 0.0,
            texture_weight: 0.42,
            normal_map_strength: 1.0,
            uv_scale: 1.0,
        }
    }

    /// Configure AO, clearcoat et sheen.
    pub fn with_layers(mut self, ambient_occlusion: f64, clearcoat: f64, sheen: Vec3) -> Self {
        self.ambient_occlusion = ambient_occlusion.clamp(0.0, 1.0);
        self.clearcoat = clearcoat.clamp(0.0, 1.0);
        self.sheen = sheen;
        self
    }

    /// Configure transmission et IOR.
    pub fn with_transmission(mut self, transmission: f64, ior: f64) -> Self {
        self.transmission = transmission.clamp(0.0, 1.0);
        self.ior = ior.max(1.0);
        self
    }

    /// Configure subsurface, anisotropie et iridescence.
    pub fn with_optics(mut self, subsurface: f64, anisotropy: f64, iridescence: f64) -> Self {
        self.subsurface = subsurface.clamp(0.0, 1.0);
        self.anisotropy = anisotropy.clamp(0.0, 1.0);
        self.iridescence = iridescence.clamp(0.0, 1.0);
        self
    }

    /// Configure les paramètres de texturing procédural.
    pub fn with_texturing(mut self, texture_weight: f64, normal_map_strength: f64, uv_scale: f64) -> Self {
        self.texture_weight = texture_weight.clamp(0.0, 1.0);
        self.normal_map_strength = normal_map_strength.clamp(0.0, 3.0);
        self.uv_scale = uv_scale.max(0.05);
        self
    }

    /// Sélectionne une texture procédurale adaptée au matériau.
    pub fn surface_texture(self) -> ProceduralTexture {
        if self.transmission > 0.18 {
            ProceduralTexture::frozen_crystal()
        } else if self.metallic > 0.65 {
            ProceduralTexture::brushed_space_metal()
        } else if self.clearcoat > 0.25 && self.roughness < 0.25 {
            ProceduralTexture::oceanic_surface()
        } else {
            ProceduralTexture::rocky_planet(self.albedo)
        }
    }

    /// Retourne l'albedo texturé procéduralement en fonction du point/UV/LOD.
    pub fn textured_albedo(self, point: Vec3, uv: Option<(f64, f64)>, lod: LodSelection) -> Vec3 {
        let freq = lod.texture_frequency.max(1.0) * self.uv_scale;
        let marble = (point.x * freq).sin() * (point.z * freq * 0.73).cos();
        let vein = ((point.x + point.y * 0.35) * freq * 1.7).sin() * 0.5 + 0.5;
        let modulation = 0.82 + marble * 0.08 + vein * 0.06 * lod.normal_intensity;
        let texture = self.surface_texture();
        let textured = texture.sample_uv(point * (0.35 + freq * 0.08), uv, self.uv_scale);
        ((self.albedo * modulation).lerp(textured, self.texture_weight + self.clearcoat * 0.10) + self.sheen * 0.03)
            .clamp(0.0, 2.2)
    }
}

/// Résultat d'intersection rayon-surface.
#[derive(Debug, Clone, Copy)]
pub struct HitRecord {
    /// Distance paramétrique le long du rayon.
    pub distance: f64,
    /// Point d'impact.
    pub point: Vec3,
    /// Normale au point d'impact.
    pub normal: Vec3,
    /// Rayon/échelle locale de l'élément touché.
    pub radius: f64,
    /// Coordonnées UV si disponibles.
    pub uv: Option<(f64, f64)>,
    /// Matériau de la surface touchée.
    pub material: Material,
}

/// Sphère raytraçable définie par un centre, un rayon et un matériau.
#[derive(Debug, Clone, Copy)]
pub struct Sphere {
    /// Centre de la sphère en espace monde.
    pub center: Vec3,
    /// Rayon de la sphère en unités monde.
    pub radius: f64,
    /// Matériau PBR de la surface.
    pub material: Material,
}

impl Sphere {
    /// Teste l'intersection d'un rayon avec la sphère. Retourne `None` si pas d'intersection.
    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let oc = ray.origin - self.center;
        let a = ray.direction.length_squared();
        let half_b = oc.dot(ray.direction);
        let c = oc.length_squared() - self.radius * self.radius;
        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return None;
        }

        let sqrt_d = discriminant.sqrt();
        let mut root = (-half_b - sqrt_d) / a;
        if root < t_min || root > t_max {
            root = (-half_b + sqrt_d) / a;
            if root < t_min || root > t_max {
                return None;
            }
        }

        let point = ray.at(root);
        let outward_normal = (point - self.center) / self.radius;

        Some(HitRecord {
            distance: root,
            point,
            normal: outward_normal.normalize(),
            radius: self.radius,
            uv: Some(spherical_uv(outward_normal.normalize())),
            material: self.material,
        })
    }
}

/// Triangle raytraçable avec normales et coordonnées UV par sommet.
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    /// Premier sommet.
    pub a: Vec3,
    /// Deuxième sommet.
    pub b: Vec3,
    /// Troisième sommet.
    pub c: Vec3,
    /// Normale interpolée au sommet A.
    pub na: Vec3,
    /// Normale interpolée au sommet B.
    pub nb: Vec3,
    /// Normale interpolée au sommet C.
    pub nc: Vec3,
    /// Coordonnées UV au sommet A.
    pub ta: (f64, f64),
    /// Coordonnées UV au sommet B.
    pub tb: (f64, f64),
    /// Coordonnées UV au sommet C.
    pub tc: (f64, f64),
    /// Matériau PBR de la surface.
    pub material: Material,
}

/// Données sources pour construire un `Triangle` via `Triangle::new`.
#[derive(Debug, Clone, Copy)]
pub struct TrianglePatch {
    /// Positions des trois sommets.
    pub positions: [Vec3; 3],
    /// Normales des trois sommets.
    pub normals: [Vec3; 3],
    /// Coordonnées UV des trois sommets.
    pub uvs: [(f64, f64); 3],
    /// Matériau PBR de la surface.
    pub material: Material,
}

impl Triangle {
    /// Construit un `Triangle` à partir d'un `TrianglePatch` (normales normalisées).
    pub fn new(patch: TrianglePatch) -> Self {
        Self {
            a: patch.positions[0],
            b: patch.positions[1],
            c: patch.positions[2],
            na: patch.normals[0].normalize(),
            nb: patch.normals[1].normalize(),
            nc: patch.normals[2].normalize(),
            ta: patch.uvs[0],
            tb: patch.uvs[1],
            tc: patch.uvs[2],
            material: patch.material,
        }
    }

    /// Construit un triangle plat (normale de face uniforme, UVs basiques).
    pub fn flat(a: Vec3, b: Vec3, c: Vec3, material: Material) -> Self {
        let face_normal = (b - a).cross(c - a).normalize();
        Self::new(TrianglePatch {
            positions: [a, b, c],
            normals: [face_normal, face_normal, face_normal],
            uvs: [(0.0, 0.0), (1.0, 0.0), (0.5, 1.0)],
            material,
        })
    }

    /// Retourne le barycentre du triangle.
    pub fn centroid(&self) -> Vec3 {
        (self.a + self.b + self.c) / 3.0
    }

    /// Teste l'intersection d'un rayon avec le triangle (algorithme Möller–Trumbore).
    pub fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        let edge1 = self.b - self.a;
        let edge2 = self.c - self.a;
        let pvec = ray.direction.cross(edge2);
        let determinant = edge1.dot(pvec);

        if determinant.abs() <= EPSILON {
            return None;
        }

        let inverse_determinant = 1.0 / determinant;
        let tvec = ray.origin - self.a;
        let u = tvec.dot(pvec) * inverse_determinant;
        if !(0.0..=1.0).contains(&u) {
            return None;
        }

        let qvec = tvec.cross(edge1);
        let v = ray.direction.dot(qvec) * inverse_determinant;
        if v < 0.0 || (u + v) > 1.0 {
            return None;
        }

        let distance = edge2.dot(qvec) * inverse_determinant;
        if distance < t_min || distance > t_max {
            return None;
        }

        let w = (1.0 - u - v).clamp(0.0, 1.0);
        let smooth_normal = (self.na * w + self.nb * u + self.nc * v).normalize();
        let uv = (
            self.ta.0 * w + self.tb.0 * u + self.tc.0 * v,
            self.ta.1 * w + self.tb.1 * u + self.tc.1 * v,
        );

        Some(HitRecord {
            distance,
            point: ray.at(distance),
            normal: smooth_normal,
            radius: (edge1.length() + edge2.length()) * 0.25,
            uv: Some(uv),
            material: self.material,
        })
    }
}

fn spherical_uv(normal: Vec3) -> (f64, f64) {
    let u = 0.5 + normal.z.atan2(normal.x) / (2.0 * std::f64::consts::PI);
    let v = 0.5 - normal.y.asin() / std::f64::consts::PI;
    (u, v)
}

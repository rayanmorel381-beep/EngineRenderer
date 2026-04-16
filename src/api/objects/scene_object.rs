use crate::api::materials::catalog::MaterialCatalog;
use crate::core::engine::rendering::raytracing::{Material, Sphere, Triangle, Vec3};

/// A scene object that can be a sphere, a triangle, or a composite group.
///
/// AI agents build scenes by creating `SceneObject`s and adding them to a
/// [`SceneBuilder`](crate::api::scenes::builder::SceneBuilder).
#[derive(Debug, Clone)]
pub enum SceneObject {
    /// Sphère paramétrique.
    Sphere {
        /// Centre de la sphère.
        center: Vec3,
        /// Rayon de la sphère.
        radius: f64,
        /// Matériau appliqué.
        material: Material,
    },
    /// Triangle plat.
    Triangle {
        /// Sommet A.
        a: Vec3,
        /// Sommet B.
        b: Vec3,
        /// Sommet C.
        c: Vec3,
        /// Matériau appliqué.
        material: Material,
    },
    /// Groupe récursif d'objets.
    Group(Vec<SceneObject>),
}

impl SceneObject {
    /// Sphere from plain arrays.
    pub fn sphere(center: [f64; 3], radius: f64, material_name: &str) -> Self {
        Self::Sphere {
            center: Vec3::new(center[0], center[1], center[2]),
            radius: radius.max(0.01),
            material: MaterialCatalog.by_name(material_name),
        }
    }

    /// Sphere with an explicit [`Material`].
    pub fn sphere_with(center: Vec3, radius: f64, material: Material) -> Self {
        Self::Sphere {
            center,
            radius: radius.max(0.01),
            material,
        }
    }

    /// Flat triangle from plain arrays.
    pub fn triangle(a: [f64; 3], b: [f64; 3], c: [f64; 3], material_name: &str) -> Self {
        Self::Triangle {
            a: Vec3::new(a[0], a[1], a[2]),
            b: Vec3::new(b[0], b[1], b[2]),
            c: Vec3::new(c[0], c[1], c[2]),
            material: MaterialCatalog.by_name(material_name),
        }
    }

    /// Group multiple objects together (e.g. a composite "house" or "tree").
    pub fn group(objects: Vec<SceneObject>) -> Self {
        Self::Group(objects)
    }

    /// Flatten into raw engine primitives.
    pub fn into_primitives(self) -> (Vec<Sphere>, Vec<Triangle>) {
        let mut spheres = Vec::new();
        let mut triangles = Vec::new();
        self.collect(&mut spheres, &mut triangles);
        (spheres, triangles)
    }

    fn collect(self, spheres: &mut Vec<Sphere>, triangles: &mut Vec<Triangle>) {
        match self {
            Self::Sphere { center, radius, material } => {
                spheres.push(Sphere { center, radius, material });
            }
            Self::Triangle { a, b, c, material } => {
                triangles.push(Triangle::flat(a, b, c, material));
            }
            Self::Group(children) => {
                for child in children {
                    child.collect(spheres, triangles);
                }
            }
        }
    }
}

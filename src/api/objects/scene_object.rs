use crate::api::materials::catalog::MaterialCatalog;
use crate::core::engine::rendering::raytracing::{Material, Sphere, Triangle, Vec3};

#[derive(Debug, Clone)]
pub enum SceneObject {
    Sphere {
        center: Vec3,
        radius: f64,
        material: Material,
    },
    Triangle {
        a: Vec3,
        b: Vec3,
        c: Vec3,
        material: Material,
    },
    Group(Vec<SceneObject>),
}

impl SceneObject {
    pub fn sphere(center: [f64; 3], radius: f64, material_name: &str) -> Self {
        Self::Sphere {
            center: Vec3::new(center[0], center[1], center[2]),
            radius: radius.max(0.01),
            material: MaterialCatalog.by_name(material_name),
        }
    }

    pub fn sphere_with(center: Vec3, radius: f64, material: Material) -> Self {
        Self::Sphere {
            center,
            radius: radius.max(0.01),
            material,
        }
    }

    pub fn triangle(a: [f64; 3], b: [f64; 3], c: [f64; 3], material_name: &str) -> Self {
        Self::Triangle {
            a: Vec3::new(a[0], a[1], a[2]),
            b: Vec3::new(b[0], b[1], b[2]),
            c: Vec3::new(c[0], c[1], c[2]),
            material: MaterialCatalog.by_name(material_name),
        }
    }

    pub fn group(objects: Vec<SceneObject>) -> Self {
        Self::Group(objects)
    }

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

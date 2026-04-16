use crate::core::engine::rendering::raytracing::{Sphere, Triangle, Vec3};

/// Boîte englobante axis-aligned pour accélération BVH.
#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    /// Coin minimum de la boîte.
    pub min: Vec3,
    /// Coin maximum de la boîte.
    pub max: Vec3,
}

impl Aabb {
    /// Retourne une boîte vide initialisée aux extrêmes.
    pub fn empty() -> Self {
        Self {
            min: Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    /// Construit l'AABB d'une sphère.
    pub fn from_sphere(sphere: &Sphere) -> Self {
        let radius = Vec3::splat(sphere.radius);
        Self {
            min: sphere.center - radius,
            max: sphere.center + radius,
        }
    }

    /// Construit l'AABB d'un triangle.
    pub fn from_triangle(tri: &Triangle) -> Self {
        Self {
            min: Vec3::new(
                tri.a.x.min(tri.b.x).min(tri.c.x),
                tri.a.y.min(tri.b.y).min(tri.c.y),
                tri.a.z.min(tri.b.z).min(tri.c.z),
            ),
            max: Vec3::new(
                tri.a.x.max(tri.b.x).max(tri.c.x),
                tri.a.y.max(tri.b.y).max(tri.c.y),
                tri.a.z.max(tri.b.z).max(tri.c.z),
            ),
        }
    }

    /// Retourne l'union de deux AABB.
    pub fn union(self, other: Self) -> Self {
        Self {
            min: Vec3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Vec3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Étend l'AABB pour inclure un point.
    pub fn expand(&mut self, point: Vec3) {
        self.min = Vec3::new(
            self.min.x.min(point.x),
            self.min.y.min(point.y),
            self.min.z.min(point.z),
        );
        self.max = Vec3::new(
            self.max.x.max(point.x),
            self.max.y.max(point.y),
            self.max.z.max(point.z),
        );
    }

    /// Retourne le centre géométrique de l'AABB.
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Retourne la surface de l'AABB.
    pub fn surface_area(&self) -> f64 {
        let extent = self.max - self.min;
        2.0 * (extent.x * extent.y + extent.y * extent.z + extent.z * extent.x).max(0.0)
    }

    /// Retourne l'axe principal (0=x, 1=y, 2=z).
    pub fn longest_axis(&self) -> usize {
        let extent = self.max - self.min;
        if extent.x >= extent.y && extent.x >= extent.z {
            0
        } else if extent.y >= extent.z {
            1
        } else {
            2
        }
    }

    /// Teste l'intersection rayon/AABB entre `t_min` et `t_max`.
    #[inline(always)]
    pub fn hit(&self, ray_origin: Vec3, ray_inv_dir: Vec3, mut t_min: f64, mut t_max: f64) -> bool {
        let mut t0 = (self.min.x - ray_origin.x) * ray_inv_dir.x;
        let mut t1 = (self.max.x - ray_origin.x) * ray_inv_dir.x;
        if ray_inv_dir.x < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);
        if t_max <= t_min {
            return false;
        }

        t0 = (self.min.y - ray_origin.y) * ray_inv_dir.y;
        t1 = (self.max.y - ray_origin.y) * ray_inv_dir.y;
        if ray_inv_dir.y < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);
        if t_max <= t_min {
            return false;
        }

        t0 = (self.min.z - ray_origin.z) * ray_inv_dir.z;
        t1 = (self.max.z - ray_origin.z) * ray_inv_dir.z;
        if ray_inv_dir.z < 0.0 {
            std::mem::swap(&mut t0, &mut t1);
        }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);

        t_max > t_min
    }
}

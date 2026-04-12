use std::cmp::Ordering;

use super::math::Vec3;
use super::primitives::{HitRecord, Ray, EPSILON};
use super::scene::Scene;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn from_sphere(sphere: &super::primitives::Sphere) -> Self {
        let radius = Vec3::splat(sphere.radius);
        Self {
            min: sphere.center - radius,
            max: sphere.center + radius,
        }
    }

    pub fn from_triangle(tri: &super::primitives::Triangle) -> Self {
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

    #[inline(always)]
    fn hit(&self, ray: &Ray, mut t_min: f64, mut t_max: f64) -> bool {
        let inv = ray.inv_direction;

        let mut t0 = (self.min.x - ray.origin.x) * inv.x;
        let mut t1 = (self.max.x - ray.origin.x) * inv.x;
        if inv.x < 0.0 { std::mem::swap(&mut t0, &mut t1); }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);
        if t_max <= t_min { return false; }

        t0 = (self.min.y - ray.origin.y) * inv.y;
        t1 = (self.max.y - ray.origin.y) * inv.y;
        if inv.y < 0.0 { std::mem::swap(&mut t0, &mut t1); }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);
        if t_max <= t_min { return false; }

        t0 = (self.min.z - ray.origin.z) * inv.z;
        t1 = (self.max.z - ray.origin.z) * inv.z;
        if inv.z < 0.0 { std::mem::swap(&mut t0, &mut t1); }
        t_min = t_min.max(t0);
        t_max = t_max.min(t1);

        t_max > t_min
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PrimitiveRef {
    Sphere(usize),
    Triangle(usize),
}

impl PrimitiveRef {
    pub fn bbox(self, scene: &Scene) -> Aabb {
        match self {
            Self::Sphere(i) => Aabb::from_sphere(&scene.objects[i]),
            Self::Triangle(i) => Aabb::from_triangle(&scene.triangles[i]),
        }
    }

    pub fn center(self, scene: &Scene) -> Vec3 {
        match self {
            Self::Sphere(i) => scene.objects[i].center,
            Self::Triangle(i) => scene.triangles[i].centroid(),
        }
    }

    pub fn hit(self, scene: &Scene, ray: &Ray, t_min: f64, t_max: f64) -> Option<HitRecord> {
        match self {
            Self::Sphere(i) => scene.objects[i].hit(ray, t_min, t_max),
            Self::Triangle(i) => scene.triangles[i].hit(ray, t_min, t_max),
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct BvhStats {
    pub node_count: usize,
    pub leaf_count: usize,
    pub primitive_count: usize,
    pub max_depth: usize,
}

#[derive(Debug, Clone)]
pub enum BvhNode {
    Leaf { bbox: Aabb, primitives: Vec<PrimitiveRef> },
    Branch { bbox: Aabb, left: Box<BvhNode>, right: Box<BvhNode> },
}

impl BvhNode {
    pub fn build(scene: &Scene) -> Option<Self> {
        let mut primitives = (0..scene.objects.len())
            .map(PrimitiveRef::Sphere)
            .collect::<Vec<_>>();
        primitives.extend((0..scene.triangles.len()).map(PrimitiveRef::Triangle));

        if primitives.is_empty() {
            None
        } else {
            Some(Self::build_recursive(scene, primitives, 0))
        }
    }

    fn build_recursive(scene: &Scene, mut primitives: Vec<PrimitiveRef>, axis: usize) -> Self {
        if primitives.len() <= 6 {
            let bbox = primitives
                .iter()
                .map(|p| p.bbox(scene))
                .reduce(Aabb::union)
                .unwrap_or_else(|| primitives[0].bbox(scene));
            return Self::Leaf { bbox, primitives };
        }

        primitives.sort_by(|a, b| {
            a.center(scene)
                .axis(axis)
                .partial_cmp(&b.center(scene).axis(axis))
                .unwrap_or(Ordering::Equal)
        });

        let right_prims = primitives.split_off(primitives.len() / 2);
        let left = Box::new(Self::build_recursive(scene, primitives, (axis + 1) % 3));
        let right = Box::new(Self::build_recursive(scene, right_prims, (axis + 1) % 3));
        let bbox = left.bbox().union(right.bbox());

        Self::Branch { bbox, left, right }
    }

    fn bbox(&self) -> Aabb {
        match self {
            Self::Leaf { bbox, .. } | Self::Branch { bbox, .. } => *bbox,
        }
    }

    pub fn stats(&self) -> BvhStats {
        self.stats_recursive(1)
    }

    fn stats_recursive(&self, depth: usize) -> BvhStats {
        match self {
            Self::Leaf { primitives, .. } => BvhStats {
                node_count: 1,
                leaf_count: 1,
                primitive_count: primitives.len(),
                max_depth: depth,
            },
            Self::Branch { left, right, .. } => {
                let ls = left.stats_recursive(depth + 1);
                let rs = right.stats_recursive(depth + 1);
                BvhStats {
                    node_count: 1 + ls.node_count + rs.node_count,
                    leaf_count: ls.leaf_count + rs.leaf_count,
                    primitive_count: ls.primitive_count + rs.primitive_count,
                    max_depth: ls.max_depth.max(rs.max_depth),
                }
            }
        }
    }

    pub fn hit_scene(scene: &Scene, ray: &Ray, t_min: f64, t_max: f64, bvh: Option<&Self>) -> Option<HitRecord> {
        if let Some(node) = bvh {
            Self::hit_bvh(scene, ray, t_min, t_max, node)
        } else {
            scene.hit(ray, t_min, t_max)
        }
    }

    fn hit_bvh(scene: &Scene, ray: &Ray, t_min: f64, t_max: f64, root: &Self) -> Option<HitRecord> {
        let mut stack: [Option<&BvhNode>; 64] = [None; 64];
        stack[0] = Some(root);
        let mut ptr = 1usize;
        let mut closest = t_max;
        let mut result = None;

        while ptr > 0 {
            ptr -= 1;
            let node = match stack[ptr].take() {
                Some(n) => n,
                None => continue,
            };

            if !node.bbox().hit(ray, t_min, closest) {
                continue;
            }

            match node {
                BvhNode::Leaf { primitives, .. } => {
                    for prim in primitives {
                        if let Some(hit) = prim.hit(scene, ray, t_min, closest) {
                            closest = hit.distance;
                            result = Some(hit);
                        }
                    }
                }
                BvhNode::Branch { left, right, .. } => {
                    let lh = left.bbox().hit(ray, t_min, closest);
                    let rh = right.bbox().hit(ray, t_min, closest);
                    match (lh, rh) {
                        (true, true) => {
                            let ld = (left.bbox().center() - ray.origin).length_squared();
                            let rd = (right.bbox().center() - ray.origin).length_squared();
                            if ld <= rd {
                                stack[ptr] = Some(right); ptr += 1;
                                stack[ptr] = Some(left); ptr += 1;
                            } else {
                                stack[ptr] = Some(left); ptr += 1;
                                stack[ptr] = Some(right); ptr += 1;
                            }
                        }
                        (true, false) => { stack[ptr] = Some(left); ptr += 1; }
                        (false, true) => { stack[ptr] = Some(right); ptr += 1; }
                        _ => {}
                    }
                }
            }
        }

        result
    }

    pub fn any_hit(scene: &Scene, ray: &Ray, max_distance: f64, bvh: Option<&Self>) -> bool {
        if let Some(root) = bvh {
            Self::any_hit_bvh(scene, ray, max_distance, root)
        } else {
            scene.is_occluded(ray, max_distance)
        }
    }

    fn any_hit_bvh(scene: &Scene, ray: &Ray, max_distance: f64, root: &Self) -> bool {
        let mut stack: [Option<&BvhNode>; 64] = [None; 64];
        stack[0] = Some(root);
        let mut ptr = 1usize;

        while ptr > 0 {
            ptr -= 1;
            let node = match stack[ptr].take() {
                Some(n) => n,
                None => continue,
            };

            if !node.bbox().hit(ray, EPSILON, max_distance) {
                continue;
            }

            match node {
                BvhNode::Leaf { primitives, .. } => {
                    for prim in primitives {
                        if prim.hit(scene, ray, EPSILON, max_distance).is_some() {
                            return true;
                        }
                    }
                }
                BvhNode::Branch { left, right, .. } => {
                    stack[ptr] = Some(left); ptr += 1;
                    stack[ptr] = Some(right); ptr += 1;
                }
            }
        }

        false
    }
}

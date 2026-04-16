use std::{cmp::Ordering, io, io::{Read, Write}, path::Path, thread};

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
    fn hit(&self, ray: &Ray, t_min: f64, t_max: f64) -> bool {

        #[cfg(target_arch = "aarch64")]
        {
            unsafe { self.hit_neon(ray, t_min as f32, t_max as f32) }
        }
        #[cfg(not(target_arch = "aarch64"))]
        {
            let inv = ray.inv_direction;
            let mut t_min = t_min;
            let mut t_max = t_max;
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
    }

    #[cfg(target_arch = "aarch64")]
    #[target_feature(enable = "neon")]
    unsafe fn hit_neon(&self, ray: &Ray, t_min: f32, t_max: f32) -> bool {
        use std::arch::aarch64::*;
        let origin = [
            ray.origin.x as f32,
            ray.origin.y as f32,
            ray.origin.z as f32,
            0.0f32,
        ];
        let inv = [
            ray.inv_direction.x as f32,
            ray.inv_direction.y as f32,
            ray.inv_direction.z as f32,
            1.0f32,
        ];
        let amin = [
            self.min.x as f32,
            self.min.y as f32,
            self.min.z as f32,
            f32::NEG_INFINITY,
        ];
        let amax = [
            self.max.x as f32,
            self.max.y as f32,
            self.max.z as f32,
            f32::INFINITY,
        ];
        let vo = unsafe { vld1q_f32(origin.as_ptr()) };
        let vi = unsafe { vld1q_f32(inv.as_ptr()) };
        let va = unsafe { vld1q_f32(amin.as_ptr()) };
        let vb = unsafe { vld1q_f32(amax.as_ptr()) };
        let t0 = vmulq_f32(vsubq_f32(va, vo), vi);
        let t1 = vmulq_f32(vsubq_f32(vb, vo), vi);
        let t_lo = vminq_f32(t0, t1);
        let t_hi = vmaxq_f32(t0, t1);
        let enter = vgetq_lane_f32(t_lo, 0)
            .max(vgetq_lane_f32(t_lo, 1))
            .max(vgetq_lane_f32(t_lo, 2))
            .max(t_min);
        let exit = vgetq_lane_f32(t_hi, 0)
            .min(vgetq_lane_f32(t_hi, 1))
            .min(vgetq_lane_f32(t_hi, 2))
            .min(t_max);
        exit > enter
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
            let workers = thread::available_parallelism()
                .map(|n| n.get())
                .unwrap_or(1)
                .max(1);
            let parallel_depth = workers.ilog2().min(2) as usize;
            Some(Self::build_recursive(scene, primitives, 0, parallel_depth))
        }
    }

    fn build_recursive(
        scene: &Scene,
        mut primitives: Vec<PrimitiveRef>,
        axis: usize,
        parallel_depth: usize,
    ) -> Self {
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
        let next_axis = (axis + 1) % 3;
        let should_parallelize = parallel_depth > 0 && right_prims.len().saturating_add(primitives.len()) >= 2048;
        let (left, right) = if should_parallelize {
            thread::scope(|scope| {
                let handle = scope.spawn(move || Self::build_recursive(scene, right_prims, next_axis, parallel_depth - 1));
                let left = Self::build_recursive(scene, primitives, next_axis, parallel_depth - 1);
                let right = handle.join().expect("failed to build BVH branch");
                (Box::new(left), Box::new(right))
            })
        } else {
            let left = Box::new(Self::build_recursive(scene, primitives, next_axis, parallel_depth));
            let right = Box::new(Self::build_recursive(scene, right_prims, next_axis, parallel_depth));
            (left, right)
        };
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

    pub fn save_to_path(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent()
            && !parent.as_os_str().is_empty()
        {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        file.write_all(b"ERBVH1")?;
        self.write_node(&mut file)
    }

    pub fn load_from_path(path: &Path, scene: &Scene) -> io::Result<Self> {
        let mut file = std::fs::File::open(path)?;
        let mut magic = [0u8; 6];
        file.read_exact(&mut magic)?;
        if &magic != b"ERBVH1" {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid BVH cache header"));
        }
        Self::read_node(&mut file, scene)
    }

    fn write_node<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        match self {
            Self::Leaf { bbox, primitives } => {
                writer.write_all(&[0u8])?;
                write_aabb(writer, bbox)?;
                writer.write_all(&(primitives.len() as u32).to_le_bytes())?;
                for primitive in primitives {
                    match primitive {
                        PrimitiveRef::Sphere(index) => {
                            writer.write_all(&[0u8])?;
                            writer.write_all(&(*index as u32).to_le_bytes())?;
                        }
                        PrimitiveRef::Triangle(index) => {
                            writer.write_all(&[1u8])?;
                            writer.write_all(&(*index as u32).to_le_bytes())?;
                        }
                    }
                }
            }
            Self::Branch { bbox, left, right } => {
                writer.write_all(&[1u8])?;
                write_aabb(writer, bbox)?;
                left.write_node(writer)?;
                right.write_node(writer)?;
            }
        }
        Ok(())
    }

    fn read_node<R: Read>(reader: &mut R, scene: &Scene) -> io::Result<Self> {
        let mut tag = [0u8; 1];
        reader.read_exact(&mut tag)?;
        let bbox = read_aabb(reader)?;
        match tag[0] {
            0 => {
                let mut len_buf = [0u8; 4];
                reader.read_exact(&mut len_buf)?;
                let primitive_len = u32::from_le_bytes(len_buf) as usize;
                let mut primitives = Vec::with_capacity(primitive_len);
                for _ in 0..primitive_len {
                    let mut primitive_tag = [0u8; 1];
                    let mut index_buf = [0u8; 4];
                    reader.read_exact(&mut primitive_tag)?;
                    reader.read_exact(&mut index_buf)?;
                    let index = u32::from_le_bytes(index_buf) as usize;
                    let primitive = match primitive_tag[0] {
                        0 if index < scene.objects.len() => PrimitiveRef::Sphere(index),
                        1 if index < scene.triangles.len() => PrimitiveRef::Triangle(index),
                        _ => {
                            return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid BVH primitive index"));
                        }
                    };
                    primitives.push(primitive);
                }
                Ok(Self::Leaf { bbox, primitives })
            }
            1 => {
                let left = Box::new(Self::read_node(reader, scene)?);
                let right = Box::new(Self::read_node(reader, scene)?);
                Ok(Self::Branch { bbox, left, right })
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "invalid BVH node tag")),
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

fn write_vec3<W: Write>(writer: &mut W, value: Vec3) -> io::Result<()> {
    writer.write_all(&value.x.to_le_bytes())?;
    writer.write_all(&value.y.to_le_bytes())?;
    writer.write_all(&value.z.to_le_bytes())
}

fn read_vec3<R: Read>(reader: &mut R) -> io::Result<Vec3> {
    let mut x = [0u8; 8];
    let mut y = [0u8; 8];
    let mut z = [0u8; 8];
    reader.read_exact(&mut x)?;
    reader.read_exact(&mut y)?;
    reader.read_exact(&mut z)?;
    Ok(Vec3::new(
        f64::from_le_bytes(x),
        f64::from_le_bytes(y),
        f64::from_le_bytes(z),
    ))
}

fn write_aabb<W: Write>(writer: &mut W, bbox: &Aabb) -> io::Result<()> {
    write_vec3(writer, bbox.min)?;
    write_vec3(writer, bbox.max)
}

fn read_aabb<R: Read>(reader: &mut R) -> io::Result<Aabb> {
    Ok(Aabb {
        min: read_vec3(reader)?,
        max: read_vec3(reader)?,
    })
}

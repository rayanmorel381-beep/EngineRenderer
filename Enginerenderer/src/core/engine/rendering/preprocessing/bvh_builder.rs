use crate::core::engine::rendering::raytracing::{Sphere, Triangle, Vec3};

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn empty() -> Self {
        Self {
            min: Vec3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY),
            max: Vec3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY),
        }
    }

    pub fn from_sphere(sphere: &Sphere) -> Self {
        let radius = Vec3::splat(sphere.radius);
        Self {
            min: sphere.center - radius,
            max: sphere.center + radius,
        }
    }

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

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn surface_area(&self) -> f64 {
        let extent = self.max - self.min;
        2.0 * (extent.x * extent.y + extent.y * extent.z + extent.z * extent.x).max(0.0)
    }

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

#[derive(Debug, Clone, Copy)]
pub struct SahSplit {
    pub axis: usize,
    pub split_count: usize,
}

pub struct SahBvhBuilder {
    pub bins: usize,
}

impl SahBvhBuilder {
    pub fn new(bins: usize) -> Self {
        Self { bins: bins.max(2) }
    }

    pub fn find_best_split(&self, bboxes: &[Aabb], centroids: &[Vec3]) -> Option<SahSplit> {
        if bboxes.len() < 2 || bboxes.len() != centroids.len() {
            return None;
        }

        let parent_bbox = bboxes.iter().copied().reduce(Aabb::union)?;
        let parent_sa = parent_bbox.surface_area().max(f64::EPSILON);
        let intersect_cost = 2.0_f64;
        let traverse_cost = 1.0_f64;
        let leaf_cost = bboxes.len() as f64 * intersect_cost;

        let mut best_cost = leaf_cost;
        let mut best_split: Option<SahSplit> = None;

        for axis in 0..3 {
            let extent_min = centroids
                .iter()
                .map(|c| c.axis(axis))
                .fold(f64::INFINITY, f64::min);
            let extent_max = centroids
                .iter()
                .map(|c| c.axis(axis))
                .fold(f64::NEG_INFINITY, f64::max);
            let extent = extent_max - extent_min;
            if extent < f64::EPSILON {
                continue;
            }

            let mut bin_bboxes = vec![Aabb::empty(); self.bins];
            let mut bin_counts = vec![0usize; self.bins];
            for (bbox, centroid) in bboxes.iter().zip(centroids.iter()) {
                let t = ((centroid.axis(axis) - extent_min) / extent).clamp(0.0, 0.9999);
                let bin = (t * self.bins as f64) as usize;
                bin_bboxes[bin] = bin_bboxes[bin].union(*bbox);
                bin_counts[bin] += 1;
            }

            let mut prefix_bboxes = vec![Aabb::empty(); self.bins];
            let mut prefix_counts = vec![0usize; self.bins];
            let mut running_bbox = Aabb::empty();
            let mut running_count = 0usize;
            for i in 0..self.bins {
                running_bbox = running_bbox.union(bin_bboxes[i]);
                running_count += bin_counts[i];
                prefix_bboxes[i] = running_bbox;
                prefix_counts[i] = running_count;
            }

            let mut suffix_bboxes = vec![Aabb::empty(); self.bins];
            let mut suffix_counts = vec![0usize; self.bins];
            running_bbox = Aabb::empty();
            running_count = 0;
            for i in (0..self.bins).rev() {
                running_bbox = running_bbox.union(bin_bboxes[i]);
                running_count += bin_counts[i];
                suffix_bboxes[i] = running_bbox;
                suffix_counts[i] = running_count;
            }

            for split in 0..(self.bins - 1) {
                let lc = prefix_counts[split];
                let rc = suffix_counts[split + 1];
                if lc == 0 || rc == 0 {
                    continue;
                }
                let ls = prefix_bboxes[split].surface_area();
                let rs = suffix_bboxes[split + 1].surface_area();
                let cost =
                    traverse_cost + (ls * lc as f64 + rs * rc as f64) * intersect_cost / parent_sa;
                if cost < best_cost {
                    best_cost = cost;
                    best_split = Some(SahSplit { axis, split_count: lc });
                }
            }
        }

        best_split
    }
}

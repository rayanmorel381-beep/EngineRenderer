use crate::core::engine::rendering::raytracing::{Scene, Vec3};

#[derive(Debug)]
pub struct WorldSdf {
    pub grid: Vec<f64>,
    pub resolution: [usize; 3],
    pub bounds_min: Vec3,
    pub cell_size: Vec3,
}

impl WorldSdf {
    pub fn build_from_scene(scene: &Scene, resolution: usize) -> Self {
        let inf = f64::INFINITY;
        let mut bounds_min = Vec3::new(inf, inf, inf);
        let mut bounds_max = Vec3::new(-inf, -inf, -inf);

        for obj in &scene.objects {
            let c = obj.center;
            let r = obj.radius;
            bounds_min.x = bounds_min.x.min(c.x - r);
            bounds_min.y = bounds_min.y.min(c.y - r);
            bounds_min.z = bounds_min.z.min(c.z - r);
            bounds_max.x = bounds_max.x.max(c.x + r);
            bounds_max.y = bounds_max.y.max(c.y + r);
            bounds_max.z = bounds_max.z.max(c.z + r);
        }
        for tri in &scene.triangles {
            for v in [tri.a, tri.b, tri.c] {
                bounds_min.x = bounds_min.x.min(v.x);
                bounds_min.y = bounds_min.y.min(v.y);
                bounds_min.z = bounds_min.z.min(v.z);
                bounds_max.x = bounds_max.x.max(v.x);
                bounds_max.y = bounds_max.y.max(v.y);
                bounds_max.z = bounds_max.z.max(v.z);
            }
        }

        if bounds_min.x == inf {
            bounds_min = Vec3::new(-10.0, -10.0, -10.0);
            bounds_max = Vec3::new(10.0, 10.0, 10.0);
        }

        let pad = Vec3::new(1.0, 1.0, 1.0);
        bounds_min = bounds_min - pad;
        bounds_max += pad;

        let span = bounds_max - bounds_min;
        let cell_size = Vec3::new(
            span.x / resolution as f64,
            span.y / resolution as f64,
            span.z / resolution as f64,
        );

        let total = resolution * resolution * resolution;
        let mut grid = vec![f64::MAX; total];

        for iz in 0..resolution {
            for iy in 0..resolution {
                for ix in 0..resolution {
                    let p = Vec3::new(
                        bounds_min.x + (ix as f64 + 0.5) * cell_size.x,
                        bounds_min.y + (iy as f64 + 0.5) * cell_size.y,
                        bounds_min.z + (iz as f64 + 0.5) * cell_size.z,
                    );
                    let mut min_dist = f64::MAX;

                    for obj in &scene.objects {
                        let d = (p - obj.center).length() - obj.radius;
                        if d < min_dist {
                            min_dist = d;
                        }
                    }

                    for tri in &scene.triangles {
                        let d = point_triangle_dist(p, tri.a, tri.b, tri.c);
                        if d < min_dist {
                            min_dist = d;
                        }
                    }

                    let idx = iz * resolution * resolution + iy * resolution + ix;
                    grid[idx] = min_dist;
                }
            }
        }

        Self {
            grid,
            resolution: [resolution, resolution, resolution],
            bounds_min,
            cell_size,
        }
    }

    pub fn sample(&self, pos: Vec3) -> f64 {
        let [rx, ry, rz] = self.resolution;
        let local = pos - self.bounds_min;
        let fx = (local.x / self.cell_size.x - 0.5).clamp(0.0, (rx - 1) as f64);
        let fy = (local.y / self.cell_size.y - 0.5).clamp(0.0, (ry - 1) as f64);
        let fz = (local.z / self.cell_size.z - 0.5).clamp(0.0, (rz - 1) as f64);

        let ix = fx as usize;
        let iy = fy as usize;
        let iz = fz as usize;
        let tx = fx - ix as f64;
        let ty = fy - iy as f64;
        let tz = fz - iz as f64;

        let ix1 = (ix + 1).min(rx - 1);
        let iy1 = (iy + 1).min(ry - 1);
        let iz1 = (iz + 1).min(rz - 1);

        let idx = |x: usize, y: usize, z: usize| z * ry * rx + y * rx + x;
        let c000 = self.grid[idx(ix, iy, iz)];
        let c100 = self.grid[idx(ix1, iy, iz)];
        let c010 = self.grid[idx(ix, iy1, iz)];
        let c110 = self.grid[idx(ix1, iy1, iz)];
        let c001 = self.grid[idx(ix, iy, iz1)];
        let c101 = self.grid[idx(ix1, iy, iz1)];
        let c011 = self.grid[idx(ix, iy1, iz1)];
        let c111 = self.grid[idx(ix1, iy1, iz1)];

        let lerp = |a: f64, b: f64, t: f64| a + (b - a) * t;
        let c00 = lerp(c000, c100, tx);
        let c01 = lerp(c001, c101, tx);
        let c10 = lerp(c010, c110, tx);
        let c11 = lerp(c011, c111, tx);
        let c0 = lerp(c00, c10, ty);
        let c1 = lerp(c01, c11, ty);
        lerp(c0, c1, tz)
    }

    pub fn gradient(&self, pos: Vec3) -> Vec3 {
        let eps = self.cell_size.x.min(self.cell_size.y).min(self.cell_size.z) * 0.5;
        let dx = self.sample(Vec3::new(pos.x + eps, pos.y, pos.z))
            - self.sample(Vec3::new(pos.x - eps, pos.y, pos.z));
        let dy = self.sample(Vec3::new(pos.x, pos.y + eps, pos.z))
            - self.sample(Vec3::new(pos.x, pos.y - eps, pos.z));
        let dz = self.sample(Vec3::new(pos.x, pos.y, pos.z + eps))
            - self.sample(Vec3::new(pos.x, pos.y, pos.z - eps));
        let g = Vec3::new(dx, dy, dz) * (1.0 / (2.0 * eps));
        let len = g.length();
        if len > f64::EPSILON {
            g * (1.0 / len)
        } else {
            Vec3::new(0.0, 1.0, 0.0)
        }
    }

    pub fn march(
        &self,
        origin: Vec3,
        direction: Vec3,
        max_t: f64,
        max_steps: u32,
    ) -> Option<(f64, Vec3)> {
        let dir = {
            let len = direction.length();
            if len > f64::EPSILON {
                direction * (1.0 / len)
            } else {
                return None;
            }
        };
        let mut t = 0.0_f64;
        for _ in 0..max_steps {
            let p = origin + dir * t;
            let d = self.sample(p);
            if d < 1e-4 {
                return Some((t, self.gradient(p)));
            }
            t += d.max(1e-4);
            if t >= max_t {
                break;
            }
        }
        None
    }

    pub fn cell_count(&self) -> usize {
        self.resolution[0] * self.resolution[1] * self.resolution[2]
    }

    pub fn sample_irradiance_hint(&self, pos: Vec3, probe_spacing: f64) -> f64 {
        let d = self.sample(pos);
        (1.0 - (d / probe_spacing).clamp(0.0, 1.0)).max(0.0)
    }
}

fn point_triangle_dist(p: Vec3, a: Vec3, b: Vec3, c: Vec3) -> f64 {
    let ab = b - a;
    let ac = c - a;
    let ap = p - a;
    let d1 = ab.dot(ap);
    let d2 = ac.dot(ap);
    if d1 <= 0.0 && d2 <= 0.0 {
        return (p - a).length();
    }
    let bp = p - b;
    let d3 = ab.dot(bp);
    let d4 = ac.dot(bp);
    if d3 >= 0.0 && d4 <= d3 {
        return (p - b).length();
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return (p - (a + ab * v)).length();
    }
    let cp = p - c;
    let d5 = ab.dot(cp);
    let d6 = ac.dot(cp);
    if d6 >= 0.0 && d5 <= d6 {
        return (p - c).length();
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return (p - (a + ac * w)).length();
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return (p - (b + (c - b) * w)).length();
    }
    let denom = 1.0 / (va + vb + vc);
    let v = vb * denom;
    let w = vc * denom;
    (p - (a + ab * v + ac * w)).length()
}

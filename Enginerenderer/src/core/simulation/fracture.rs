use crate::core::engine::rendering::raytracing::Vec3;

#[derive(Debug, Clone)]
pub struct VoronoiCell {
    pub center: Vec3,
    pub vertices: Vec<Vec3>,
    pub broken: bool,
}

#[derive(Debug, Clone)]
pub struct Bond {
    pub a: usize,
    pub b: usize,
    pub strength: f64,
    pub broken: bool,
}

pub struct FractureBody {
    pub cells: Vec<VoronoiCell>,
    pub bonds: Vec<Bond>,
    pub threshold: f64,
}

impl FractureBody {
    pub fn generate(bounds_min: Vec3, bounds_max: Vec3, seed_count: usize, seed: u64) -> Self {
        let centers = lcg_points(bounds_min, bounds_max, seed_count, seed);
        let cells: Vec<VoronoiCell> = centers.into_iter().map(|c| VoronoiCell {
            center: c,
            vertices: Vec::new(),
            broken: false,
        }).collect();

        let n = cells.len();
        let mut bonds = Vec::new();
        for i in 0..n {
            for j in i + 1..n {
                let dist = (cells[i].center - cells[j].center).length();
                let extent = (bounds_max - bounds_min).length();
                let neighbor_threshold = extent / (seed_count as f64).sqrt() * 1.5;
                if dist < neighbor_threshold {
                    bonds.push(Bond { a: i, b: j, strength: 1.0, broken: false });
                }
            }
        }

        Self { cells, bonds, threshold: 0.5 }
    }

    pub fn apply_impulse(&mut self, point: Vec3, force_magnitude: f64) {
        for bond in &mut self.bonds {
            if bond.broken { continue; }
            let ca = self.cells[bond.a].center;
            let cb = self.cells[bond.b].center;
            let midpoint = (ca + cb) * 0.5;
            let dist = (midpoint - point).length().max(1e-6);
            let stress = force_magnitude / (dist * dist);
            if stress > bond.strength * self.threshold {
                bond.broken = true;
            }
        }

        for cell in &mut self.cells {
            let dist = (cell.center - point).length().max(1e-6);
            let stress = force_magnitude / (dist * dist);
            if stress > self.threshold * 2.0 {
                cell.broken = true;
            }
        }
    }

    pub fn fractured_cells(&self) -> Vec<&VoronoiCell> {
        self.cells.iter().filter(|c| c.broken).collect()
    }

    pub fn intact_cells(&self) -> Vec<&VoronoiCell> {
        self.cells.iter().filter(|c| !c.broken).collect()
    }

    pub fn is_fully_fractured(&self) -> bool {
        self.bonds.iter().all(|b| b.broken)
    }

    pub fn intact_bond_count(&self) -> usize {
        self.bonds.iter().filter(|b| !b.broken).count()
    }
}

fn lcg_points(min: Vec3, max: Vec3, count: usize, seed: u64) -> Vec<Vec3> {
    let mut s = seed.wrapping_add(0x0DDF_ACC1_7777_7777);
    let mut points = Vec::with_capacity(count);
    let range = max - min;
    for _ in 0..count {
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let fx = ((s >> 33) as f64) / (u32::MAX as f64);
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let fy = ((s >> 33) as f64) / (u32::MAX as f64);
        s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let fz = ((s >> 33) as f64) / (u32::MAX as f64);
        points.push(Vec3::new(
            min.x + fx * range.x,
            min.y + fy * range.y,
            min.z + fz * range.z,
        ));
    }
    points
}

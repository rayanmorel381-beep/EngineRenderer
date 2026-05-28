use crate::core::engine::rendering::raytracing::Vec3;
use std::collections::HashMap;

pub struct SpatialHash {
    pub cells: HashMap<[i32; 3], Vec<usize>>,
    pub cell_size: f64,
}

impl SpatialHash {
    pub fn new(cell_size: f64) -> Self {
        Self { cells: HashMap::new(), cell_size }
    }

    pub fn clear(&mut self) {
        self.cells.clear();
    }

    pub fn insert(&mut self, id: usize, aabb_min: Vec3, aabb_max: Vec3) {
        let min_cell = cell_coord(aabb_min, self.cell_size);
        let max_cell = cell_coord(aabb_max, self.cell_size);
        for z in min_cell[2]..=max_cell[2] {
            for y in min_cell[1]..=max_cell[1] {
                for x in min_cell[0]..=max_cell[0] {
                    self.cells.entry([x, y, z]).or_default().push(id);
                }
            }
        }
    }

    pub fn query(&self, aabb_min: Vec3, aabb_max: Vec3) -> Vec<usize> {
        let min_cell = cell_coord(aabb_min, self.cell_size);
        let max_cell = cell_coord(aabb_max, self.cell_size);
        let mut result = Vec::new();
        for z in min_cell[2]..=max_cell[2] {
            for y in min_cell[1]..=max_cell[1] {
                for x in min_cell[0]..=max_cell[0] {
                    if let Some(ids) = self.cells.get(&[x, y, z]) {
                        for &id in ids {
                            if !result.contains(&id) {
                                result.push(id);
                            }
                        }
                    }
                }
            }
        }
        result
    }

    pub fn query_pairs(&self) -> Vec<(usize, usize)> {
        let mut pairs = Vec::new();
        for ids in self.cells.values() {
            let n = ids.len();
            for i in 0..n {
                for j in i + 1..n {
                    let a = ids[i].min(ids[j]);
                    let b = ids[i].max(ids[j]);
                    if !pairs.contains(&(a, b)) {
                        pairs.push((a, b));
                    }
                }
            }
        }
        pairs
    }

    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }
}

pub fn cell_coord(pos: Vec3, cell_size: f64) -> [i32; 3] {
    [
        (pos.x / cell_size).floor() as i32,
        (pos.y / cell_size).floor() as i32,
        (pos.z / cell_size).floor() as i32,
    ]
}

#[derive(Clone, Debug)]
pub struct ClothVertex {
    pub position: [f64; 3],
    pub prev_position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass_inv: f64,
    pub pinned: bool,
}

impl ClothVertex {
    pub fn new(pos: [f64; 3]) -> Self {
        Self { position: pos, prev_position: pos, velocity: [0.0; 3], mass_inv: 1.0, pinned: false }
    }
}

#[derive(Clone, Debug)]
pub struct ClothConstraint {
    pub a: usize,
    pub b: usize,
    pub rest_length: f64,
    pub compliance: f64,
}

#[derive(Clone, Debug)]
pub struct ClothMesh {
    pub vertices: Vec<ClothVertex>,
    pub distance_constraints: Vec<ClothConstraint>,
    pub bend_constraints: Vec<ClothConstraint>,
    pub gravity: [f64; 3],
    pub wind: [f64; 3],
    pub damping: f64,
    pub substeps: usize,
    pub stretch_compliance: f64,
    pub bend_compliance: f64,
}

impl ClothMesh {
    pub fn new_grid(rows: usize, cols: usize, cell_size: f64) -> Self {
        let mut verts = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let mut v = ClothVertex::new([c as f64 * cell_size, 0.0, r as f64 * cell_size]);
                if r == 0 { v.pinned = true; v.mass_inv = 0.0; }
                verts.push(v);
            }
        }
        let mut dist_c = Vec::new();
        let mut bend_c = Vec::new();
        for r in 0..rows {
            for c in 0..cols {
                let i = r * cols + c;
                if c + 1 < cols {
                    let len = cell_size;
                    dist_c.push(ClothConstraint { a: i, b: i+1, rest_length: len, compliance: 0.0 });
                }
                if r + 1 < rows {
                    let len = cell_size;
                    dist_c.push(ClothConstraint { a: i, b: (r+1)*cols+c, rest_length: len, compliance: 0.0 });
                }
                if c + 2 < cols {
                    bend_c.push(ClothConstraint { a: i, b: i+2, rest_length: cell_size*2.0, compliance: 1e-3 });
                }
                if r + 2 < rows {
                    bend_c.push(ClothConstraint { a: i, b: (r+2)*cols+c, rest_length: cell_size*2.0, compliance: 1e-3 });
                }
            }
        }
        Self {
            vertices: verts,
            distance_constraints: dist_c,
            bend_constraints: bend_c,
            gravity: [0.0, -9.81, 0.0],
            wind: [0.0; 3],
            damping: 0.01,
            substeps: 4,
            stretch_compliance: 0.0,
            bend_compliance: 1e-3,
        }
    }

    pub fn step(&mut self, dt: f64) {
        let sub_dt = dt / self.substeps as f64;
        for _ in 0..self.substeps {
            for v in self.vertices.iter_mut() {
                if v.pinned { continue; }
                let ax = self.gravity[0] + self.wind[0];
                let ay = self.gravity[1] + self.wind[1];
                let az = self.gravity[2] + self.wind[2];
                v.prev_position = v.position;
                v.position[0] += v.velocity[0] * sub_dt + ax * sub_dt * sub_dt;
                v.position[1] += v.velocity[1] * sub_dt + ay * sub_dt * sub_dt;
                v.position[2] += v.velocity[2] * sub_dt + az * sub_dt * sub_dt;
            }
            let all_c: Vec<ClothConstraint> = self.distance_constraints.iter().chain(self.bend_constraints.iter()).cloned().collect();
            for c in &all_c {
                let (pa, pb) = {
                    let a = &self.vertices[c.a];
                    let b = &self.vertices[c.b];
                    (a.position, b.position)
                };
                let dx = [pb[0]-pa[0], pb[1]-pa[1], pb[2]-pa[2]];
                let dist = (dx[0]*dx[0]+dx[1]*dx[1]+dx[2]*dx[2]).sqrt().max(1e-9);
                let diff = (dist - c.rest_length) / (dist * (self.vertices[c.a].mass_inv + self.vertices[c.b].mass_inv) + c.compliance / (sub_dt*sub_dt));
                let corr = [dx[0]/dist*diff, dx[1]/dist*diff, dx[2]/dist*diff];
                if !self.vertices[c.a].pinned {
                    let mi = self.vertices[c.a].mass_inv;
                    self.vertices[c.a].position[0] += corr[0] * mi;
                    self.vertices[c.a].position[1] += corr[1] * mi;
                    self.vertices[c.a].position[2] += corr[2] * mi;
                }
                if !self.vertices[c.b].pinned {
                    let mi = self.vertices[c.b].mass_inv;
                    self.vertices[c.b].position[0] -= corr[0] * mi;
                    self.vertices[c.b].position[1] -= corr[1] * mi;
                    self.vertices[c.b].position[2] -= corr[2] * mi;
                }
                if let Some(v) = self.vertices[c.a].position.get_mut(1) { if *v < 0.0 { *v = 0.0; } }
                if let Some(v) = self.vertices[c.b].position.get_mut(1) { if *v < 0.0 { *v = 0.0; } }
            }
            let damp = 1.0 - self.damping;
            for v in self.vertices.iter_mut() {
                if v.pinned { continue; }
                v.velocity[0] = (v.position[0] - v.prev_position[0]) / sub_dt * damp;
                v.velocity[1] = (v.position[1] - v.prev_position[1]) / sub_dt * damp;
                v.velocity[2] = (v.position[2] - v.prev_position[2]) / sub_dt * damp;
            }
        }
    }
}

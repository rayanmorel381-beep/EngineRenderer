#[derive(Clone, Debug)]
pub struct SoftVertex {
    pub position: [f64; 3],
    pub prev_position: [f64; 3],
    pub velocity: [f64; 3],
    pub mass_inv: f64,
    pub pinned: bool,
}

impl SoftVertex {
    pub fn new(pos: [f64; 3]) -> Self {
        Self { position: pos, prev_position: pos, velocity: [0.0; 3], mass_inv: 1.0, pinned: false }
    }
}

#[derive(Clone, Debug)]
pub struct SoftConstraint {
    pub a: usize,
    pub b: usize,
    pub rest_length: f64,
    pub stiffness: f64,
    pub is_volume: bool,
}

#[derive(Clone, Debug)]
pub struct SoftBody {
    pub vertices: Vec<SoftVertex>,
    pub constraints: Vec<SoftConstraint>,
    pub gravity: [f64; 3],
    pub damping: f64,
    pub pressure: f64,
    pub substeps: usize,
}

impl Default for SoftBody {
    fn default() -> Self {
        Self {
            vertices: Vec::new(),
            constraints: Vec::new(),
            gravity: [0.0, -9.81, 0.0],
            damping: 0.02,
            pressure: 1.0,
            substeps: 4,
        }
    }
}

impl SoftBody {
    pub fn new() -> Self { Self::default() }

    pub fn from_box(half: f64) -> Self {
        let corners = [
            [-half, -half, -half], [ half, -half, -half], [-half,  half, -half], [ half,  half, -half],
            [-half, -half,  half], [ half, -half,  half], [-half,  half,  half], [ half,  half,  half],
        ];
        let mut sb = Self::new();
        for c in &corners { sb.vertices.push(SoftVertex::new(*c)); }
        for i in 0..8usize {
            for j in (i+1)..8usize {
                let p = sb.vertices[i].position;
                let q = sb.vertices[j].position;
                let d = ((q[0]-p[0]).powi(2)+(q[1]-p[1]).powi(2)+(q[2]-p[2]).powi(2)).sqrt();
                sb.constraints.push(SoftConstraint { a: i, b: j, rest_length: d, stiffness: 0.9, is_volume: false });
            }
        }
        sb
    }

    pub fn step(&mut self, dt: f64) {
        let sub_dt = dt / self.substeps as f64;
        for _ in 0..self.substeps {
            for v in self.vertices.iter_mut() {
                if v.pinned { continue; }
                v.prev_position = v.position;
                for axis in 0..3 {
                    v.position[axis] += v.velocity[axis] * sub_dt + self.gravity[axis] * sub_dt * sub_dt;
                }
            }
            let constraints = self.constraints.clone();
            for c in &constraints {
                let pa = self.vertices[c.a].position;
                let pb = self.vertices[c.b].position;
                let dx = [pb[0]-pa[0], pb[1]-pa[1], pb[2]-pa[2]];
                let dist = (dx[0]*dx[0]+dx[1]*dx[1]+dx[2]*dx[2]).sqrt().max(1e-9);
                let diff = (dist - c.rest_length) * c.stiffness
                    / (dist * (self.vertices[c.a].mass_inv + self.vertices[c.b].mass_inv).max(1e-9));
                let corr = [dx[0]/dist*diff, dx[1]/dist*diff, dx[2]/dist*diff];
                if !self.vertices[c.a].pinned {
                    let mi = self.vertices[c.a].mass_inv;
                    for ax in 0..3 { self.vertices[c.a].position[ax] += corr[ax] * mi; }
                }
                if !self.vertices[c.b].pinned {
                    let mi = self.vertices[c.b].mass_inv;
                    for ax in 0..3 { self.vertices[c.b].position[ax] -= corr[ax] * mi; }
                }
                if let Some(y) = self.vertices[c.a].position.get_mut(1) { if *y < 0.0 { *y = 0.0; } }
                if let Some(y) = self.vertices[c.b].position.get_mut(1) { if *y < 0.0 { *y = 0.0; } }
            }
            let damp = 1.0 - self.damping;
            for v in self.vertices.iter_mut() {
                if v.pinned { continue; }
                for ax in 0..3 { v.velocity[ax] = (v.position[ax] - v.prev_position[ax]) / sub_dt * damp; }
            }
        }
    }
}

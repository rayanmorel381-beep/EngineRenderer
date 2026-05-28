fn vec3_sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn vec3_add(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn vec3_len(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}
fn vec3_norm(v: [f64; 3]) -> [f64; 3] {
    let l = vec3_len(v).max(1e-10);
    [v[0] / l, v[1] / l, v[2] / l]
}
fn vec3_scale(v: [f64; 3], s: f64) -> [f64; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}

#[derive(Clone, Debug)]
pub enum IkKind {
    Fabrik,
    TwoBone,
}

impl Default for IkKind {
    fn default() -> Self { Self::Fabrik }
}

impl IkKind {
    pub fn label(&self) -> &'static str {
        match self { Self::Fabrik => "FABRIK", Self::TwoBone => "Two-Bone" }
    }
    pub const ALL: [IkKind; 2] = [IkKind::Fabrik, IkKind::TwoBone];
}

#[derive(Clone, Debug)]
pub struct IkChain {
    pub name: String,
    pub kind: IkKind,
    pub bone_indices: Vec<usize>,
    pub target: [f64; 3],
    pub pole_target: Option<[f64; 3]>,
    pub max_iterations: usize,
    pub tolerance: f64,
    pub weight: f64,
    pub enabled: bool,
}

impl Default for IkChain {
    fn default() -> Self {
        Self {
            name: "IK Chain".to_string(),
            kind: IkKind::Fabrik,
            bone_indices: Vec::new(),
            target: [0.0, 0.0, 0.0],
            pole_target: None,
            max_iterations: 10,
            tolerance: 0.001,
            weight: 1.0,
            enabled: true,
        }
    }
}

impl IkChain {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    pub fn solve(&self, positions: &mut Vec<[f64; 3]>) {
        if self.bone_indices.is_empty() || positions.is_empty() { return; }
        match self.kind {
            IkKind::Fabrik => self.solve_fabrik(positions),
            IkKind::TwoBone => self.solve_two_bone(positions),
        }
    }

    fn solve_fabrik(&self, positions: &mut Vec<[f64; 3]>) {
        let n = self.bone_indices.len().min(positions.len());
        if n < 2 { return; }
        let mut chain: Vec<[f64; 3]> = self.bone_indices.iter().take(n).map(|&i| positions[i]).collect();
        let root = chain[0];
        let mut lengths = Vec::with_capacity(n - 1);
        for i in 0..n - 1 {
            lengths.push(vec3_len(vec3_sub(chain[i + 1], chain[i])));
        }
        let total_len: f64 = lengths.iter().sum();
        let dist_to_target = vec3_len(vec3_sub(self.target, root));
        if dist_to_target >= total_len {
            let dir = vec3_norm(vec3_sub(self.target, root));
            let mut acc = root;
            for i in 0..n - 1 {
                chain[i] = acc;
                acc = vec3_add(acc, vec3_scale(dir, lengths[i]));
            }
            chain[n - 1] = acc;
        } else {
            for _ in 0..self.max_iterations {
                chain[n - 1] = self.target;
                for i in (0..n - 1).rev() {
                    let dir = vec3_norm(vec3_sub(chain[i], chain[i + 1]));
                    chain[i] = vec3_add(chain[i + 1], vec3_scale(dir, lengths[i]));
                }
                chain[0] = root;
                for i in 0..n - 1 {
                    let dir = vec3_norm(vec3_sub(chain[i + 1], chain[i]));
                    chain[i + 1] = vec3_add(chain[i], vec3_scale(dir, lengths[i]));
                }
                if vec3_len(vec3_sub(chain[n - 1], self.target)) < self.tolerance { break; }
            }
        }
        for (k, &bi) in self.bone_indices.iter().take(n).enumerate() {
            if bi < positions.len() {
                positions[bi] = chain[k];
            }
        }
    }

    fn solve_two_bone(&self, positions: &mut Vec<[f64; 3]>) {
        if self.bone_indices.len() < 3 { return; }
        let (i0, i1, i2) = (self.bone_indices[0], self.bone_indices[1], self.bone_indices[2]);
        if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() { return; }
        let a = positions[i0];
        let c = self.target;
        let lab = vec3_len(vec3_sub(positions[i1], a));
        let lbc = vec3_len(vec3_sub(positions[i2], positions[i1]));
        let lac = vec3_len(vec3_sub(c, a)).min(lab + lbc - 1e-6);
        let cos_a = ((lab * lab + lac * lac - lbc * lbc) / (2.0 * lab * lac)).clamp(-1.0, 1.0);
        let angle_a = cos_a.acos();
        let ac_dir = vec3_norm(vec3_sub(c, a));
        let perp = if let Some(pole) = self.pole_target {
            let to_pole = vec3_norm(vec3_sub(pole, a));
            let proj_len = to_pole[0] * ac_dir[0] + to_pole[1] * ac_dir[1] + to_pole[2] * ac_dir[2];
            let proj = vec3_scale(ac_dir, proj_len);
            vec3_norm(vec3_sub(to_pole, proj))
        } else {
            let arb = if ac_dir[1].abs() < 0.9 { [0.0, 1.0, 0.0] } else { [1.0, 0.0, 0.0] };
            let proj_len = arb[0] * ac_dir[0] + arb[1] * ac_dir[1] + arb[2] * ac_dir[2];
            let proj = vec3_scale(ac_dir, proj_len);
            vec3_norm(vec3_sub(arb, proj))
        };
        let mid_dir = vec3_add(vec3_scale(ac_dir, angle_a.cos()), vec3_scale(perp, angle_a.sin()));
        positions[i1] = vec3_add(a, vec3_scale(mid_dir, lab));
        positions[i2] = c;
    }
}

#[derive(Clone, Debug, Default)]
pub struct IkRig {
    pub chains: Vec<IkChain>,
}

impl IkRig {
    pub fn new() -> Self { Self::default() }

    pub fn add_chain(&mut self, chain: IkChain) -> usize {
        let idx = self.chains.len();
        self.chains.push(chain);
        idx
    }

    pub fn remove_chain(&mut self, index: usize) {
        if index < self.chains.len() { self.chains.remove(index); }
    }

    pub fn solve_all(&self, positions: &mut Vec<[f64; 3]>) {
        for chain in &self.chains {
            if chain.enabled {
                chain.solve(positions);
            }
        }
    }
}

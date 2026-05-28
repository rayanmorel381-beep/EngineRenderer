fn lerp3(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

#[derive(Clone, Debug, PartialEq)]
pub enum SplineKind {
    Bezier,
    CatmullRom,
    Linear,
}

impl Default for SplineKind {
    fn default() -> Self { Self::CatmullRom }
}

impl SplineKind {
    pub fn label(&self) -> &'static str {
        match self { Self::Bezier => "Bézier", Self::CatmullRom => "Catmull-Rom", Self::Linear => "Linéaire" }
    }
    pub const ALL: [SplineKind; 3] = [SplineKind::Bezier, SplineKind::CatmullRom, SplineKind::Linear];
}

#[derive(Clone, Debug)]
pub struct SplinePoint {
    pub position: [f64; 3],
    pub tangent_in: [f64; 3],
    pub tangent_out: [f64; 3],
    pub roll: f64,
    pub scale: f64,
}

impl SplinePoint {
    pub fn new(position: [f64; 3]) -> Self {
        Self { position, tangent_in: [0.0, 0.0, -1.0], tangent_out: [0.0, 0.0, 1.0], roll: 0.0, scale: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct Spline {
    pub name: String,
    pub kind: SplineKind,
    pub points: Vec<SplinePoint>,
    pub closed: bool,
    pub resolution: usize,
}

impl Default for Spline {
    fn default() -> Self {
        let mut s = Self {
            name: "Spline".to_string(),
            kind: SplineKind::CatmullRom,
            points: Vec::new(),
            closed: false,
            resolution: 20,
        };
        s.points.push(SplinePoint::new([0.0, 0.0, 0.0]));
        s.points.push(SplinePoint::new([1.0, 0.0, 0.0]));
        s.points.push(SplinePoint::new([2.0, 0.5, 0.0]));
        s.points.push(SplinePoint::new([3.0, 0.0, 0.0]));
        s
    }
}

impl Spline {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Self::default() }
    }

    pub fn add_point(&mut self, position: [f64; 3]) {
        self.points.push(SplinePoint::new(position));
    }

    pub fn remove_point(&mut self, index: usize) {
        if index < self.points.len() { self.points.remove(index); }
    }

    pub fn evaluate(&self, t: f64) -> [f64; 3] {
        if self.points.is_empty() { return [0.0; 3]; }
        if self.points.len() == 1 { return self.points[0].position; }
        let n = self.points.len();
        let segment_count = if self.closed { n } else { n - 1 };
        let t = t.clamp(0.0, 1.0) * segment_count as f64;
        let seg = (t as usize).min(segment_count - 1);
        let lt = t - seg as f64;
        match self.kind {
            SplineKind::Linear => {
                let p0 = self.points[seg].position;
                let p1 = self.points[(seg + 1) % n].position;
                lerp3(p0, p1, lt)
            }
            SplineKind::Bezier => {
                let p0 = self.points[seg].position;
                let p1 = self.points[seg].tangent_out;
                let p2 = self.points[(seg + 1) % n].tangent_in;
                let p3 = self.points[(seg + 1) % n].position;
                cubic_bezier(p0, p1, p2, p3, lt)
            }
            SplineKind::CatmullRom => {
                let p0 = if seg == 0 { self.points[0].position } else { self.points[seg - 1].position };
                let p1 = self.points[seg].position;
                let p2 = self.points[(seg + 1) % n].position;
                let p3 = if seg + 2 < n { self.points[seg + 2].position } else { self.points[n - 1].position };
                catmull_rom(p0, p1, p2, p3, lt)
            }
        }
    }

    pub fn bake_points(&self) -> Vec<[f64; 3]> {
        let total = self.resolution * (self.points.len().saturating_sub(1)).max(1);
        (0..=total).map(|i| self.evaluate(i as f64 / total as f64)).collect()
    }

    pub fn length_approx(&self) -> f64 {
        let pts = self.bake_points();
        pts.windows(2).map(|w| {
            let d = [w[1][0] - w[0][0], w[1][1] - w[0][1], w[1][2] - w[0][2]];
            (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt()
        }).sum()
    }
}

fn cubic_bezier(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], t: f64) -> [f64; 3] {
    let u = 1.0 - t;
    let c0 = u * u * u;
    let c1 = 3.0 * u * u * t;
    let c2 = 3.0 * u * t * t;
    let c3 = t * t * t;
    [c0*p0[0]+c1*p1[0]+c2*p2[0]+c3*p3[0], c0*p0[1]+c1*p1[1]+c2*p2[1]+c3*p3[1], c0*p0[2]+c1*p1[2]+c2*p2[2]+c3*p3[2]]
}

fn catmull_rom(p0: [f64; 3], p1: [f64; 3], p2: [f64; 3], p3: [f64; 3], t: f64) -> [f64; 3] {
    let t2 = t * t;
    let t3 = t2 * t;
    let c0 = -t3 + 2.0*t2 - t;
    let c1 = 3.0*t3 - 5.0*t2 + 2.0;
    let c2 = -3.0*t3 + 4.0*t2 + t;
    let c3 = t3 - t2;
    [
        0.5*(c0*p0[0]+c1*p1[0]+c2*p2[0]+c3*p3[0]),
        0.5*(c0*p0[1]+c1*p1[1]+c2*p2[1]+c3*p3[1]),
        0.5*(c0*p0[2]+c1*p1[2]+c2*p2[2]+c3*p3[2]),
    ]
}

#[derive(Clone, Debug)]
pub struct SplineMeshDeformer {
    pub spline: Spline,
    pub mesh_asset: Option<String>,
    pub repeat_count: usize,
    pub scale_mode: DeformerScaleMode,
    pub enabled: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeformerScaleMode {
    Stretch,
    Tile,
}

impl DeformerScaleMode {
    pub fn label(&self) -> &'static str {
        match self { Self::Stretch => "Étirer", Self::Tile => "Tuile" }
    }
    pub const ALL: [DeformerScaleMode; 2] = [DeformerScaleMode::Stretch, DeformerScaleMode::Tile];
}

impl Default for SplineMeshDeformer {
    fn default() -> Self {
        Self { spline: Spline::default(), mesh_asset: None, repeat_count: 1, scale_mode: DeformerScaleMode::Stretch, enabled: true }
    }
}

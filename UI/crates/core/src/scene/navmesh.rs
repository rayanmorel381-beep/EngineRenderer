use std::collections::{BinaryHeap, HashMap};
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct NavPolygon {
    pub vertices: Vec<[f64; 2]>,
    pub center: [f64; 2],
    pub neighbors: Vec<usize>,
    pub walkable: bool,
    pub area_cost: f64,
}

impl NavPolygon {
    pub fn new(vertices: Vec<[f64; 2]>) -> Self {
        let cx = vertices.iter().map(|v| v[0]).sum::<f64>() / vertices.len() as f64;
        let cz = vertices.iter().map(|v| v[1]).sum::<f64>() / vertices.len() as f64;
        Self { vertices, center: [cx, cz], neighbors: Vec::new(), walkable: true, area_cost: 1.0 }
    }
}

#[derive(Clone, Debug)]
pub struct NavPath {
    pub waypoints: Vec<[f64; 3]>,
    pub complete: bool,
}

#[derive(Clone, Debug)]
pub struct NavMesh {
    pub polygons: Vec<NavPolygon>,
    pub cell_size: f64,
    pub agent_radius: f64,
    pub agent_height: f64,
    pub max_slope_deg: f64,
}

impl NavMesh {
    pub fn new() -> Self {
        Self { polygons: Vec::new(), cell_size: 0.5, agent_radius: 0.3, agent_height: 1.8, max_slope_deg: 45.0 }
    }

    pub fn build_grid(&mut self, width: usize, depth: usize) {
        self.polygons.clear();
        let cs = self.cell_size;
        for iz in 0..depth {
            for ix in 0..width {
                let x = ix as f64 * cs;
                let z = iz as f64 * cs;
                let verts = vec![[x, z], [x+cs, z], [x+cs, z+cs], [x, z+cs]];
                self.polygons.push(NavPolygon::new(verts));
            }
        }
        for iz in 0..depth {
            for ix in 0..width {
                let i = iz * width + ix;
                if ix + 1 < width { let j = iz * width + (ix+1); self.polygons[i].neighbors.push(j); self.polygons[j].neighbors.push(i); }
                if iz + 1 < depth { let j = (iz+1) * width + ix; self.polygons[i].neighbors.push(j); self.polygons[j].neighbors.push(i); }
            }
        }
    }

    pub fn find_nearest_polygon(&self, point: [f64; 3]) -> Option<usize> {
        self.polygons.iter().enumerate()
            .filter(|(_, p)| p.walkable)
            .min_by(|(_, a), (_, b)| {
                let da = (a.center[0]-point[0]).powi(2)+(a.center[1]-point[2]).powi(2);
                let db = (b.center[0]-point[0]).powi(2)+(b.center[1]-point[2]).powi(2);
                da.partial_cmp(&db).unwrap_or(Ordering::Equal)
            })
            .map(|(i, _)| i)
    }

    pub fn find_path(&self, from: [f64; 3], to: [f64; 3]) -> NavPath {
        let start = match self.find_nearest_polygon(from) { Some(i) => i, None => return NavPath { waypoints: Vec::new(), complete: false } };
        let goal = match self.find_nearest_polygon(to) { Some(i) => i, None => return NavPath { waypoints: Vec::new(), complete: false } };
        if start == goal { return NavPath { waypoints: vec![to], complete: true }; }
        let h = |i: usize| -> f64 {
            let c = &self.polygons[i].center;
            let g = &self.polygons[goal].center;
            ((c[0]-g[0]).powi(2)+(c[1]-g[1]).powi(2)).sqrt()
        };
        let mut g_score: HashMap<usize, f64> = HashMap::new();
        let mut came_from: HashMap<usize, usize> = HashMap::new();
        let mut open: BinaryHeap<AStarNode> = BinaryHeap::new();
        g_score.insert(start, 0.0);
        open.push(AStarNode { idx: start, f: h(start) });
        while let Some(AStarNode { idx: current, .. }) = open.pop() {
            if current == goal {
                let mut path = vec![to];
                let mut cur = current;
                while let Some(&prev) = came_from.get(&cur) {
                    let c = &self.polygons[cur].center;
                    path.push([c[0], 0.0, c[1]]);
                    cur = prev;
                }
                path.reverse();
                return NavPath { waypoints: path, complete: true };
            }
            let current_g = *g_score.get(&current).unwrap_or(&f64::INFINITY);
            let neighbors = self.polygons[current].neighbors.clone();
            for nb in neighbors {
                if !self.polygons[nb].walkable { continue; }
                let ca = &self.polygons[current].center;
                let cb = &self.polygons[nb].center;
                let edge = ((ca[0]-cb[0]).powi(2)+(ca[1]-cb[1]).powi(2)).sqrt();
                let tentative = current_g + edge * self.polygons[nb].area_cost;
                if tentative < *g_score.get(&nb).unwrap_or(&f64::INFINITY) {
                    g_score.insert(nb, tentative);
                    came_from.insert(nb, current);
                    open.push(AStarNode { idx: nb, f: tentative + h(nb) });
                }
            }
        }
        NavPath { waypoints: Vec::new(), complete: false }
    }
}

impl Default for NavMesh { fn default() -> Self { Self::new() } }

#[derive(Clone, Debug)]
pub struct NavAgent {
    pub position: [f64; 3],
    pub target: Option<[f64; 3]>,
    pub path: NavPath,
    pub speed: f64,
    pub stopping_distance: f64,
    pub current_waypoint: usize,
}

impl NavAgent {
    pub fn new(position: [f64; 3]) -> Self {
        Self { position, target: None, path: NavPath { waypoints: Vec::new(), complete: false }, speed: 3.5, stopping_distance: 0.2, current_waypoint: 0 }
    }

    pub fn set_destination(&mut self, target: [f64; 3], mesh: &NavMesh) {
        self.target = Some(target);
        self.path = mesh.find_path(self.position, target);
        self.current_waypoint = 0;
    }

    pub fn tick(&mut self, dt: f64) {
        if self.current_waypoint >= self.path.waypoints.len() { return; }
        let wp = self.path.waypoints[self.current_waypoint];
        let dx = wp[0] - self.position[0];
        let dz = wp[2] - self.position[2];
        let dist = (dx*dx + dz*dz).sqrt();
        if dist < self.stopping_distance {
            self.current_waypoint += 1;
        } else {
            let move_dist = (self.speed * dt).min(dist);
            self.position[0] += dx / dist * move_dist;
            self.position[2] += dz / dist * move_dist;
        }
    }

    pub fn has_arrived(&self) -> bool { self.current_waypoint >= self.path.waypoints.len() }
}

#[derive(PartialEq)]
struct AStarNode { idx: usize, f: f64 }
impl Eq for AStarNode {}
impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.f.partial_cmp(&self.f).unwrap_or(Ordering::Equal)
    }
}

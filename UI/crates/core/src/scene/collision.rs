use crate::scene::ObjectId;

#[derive(Clone, Debug)]
pub struct Aabb {
    pub min: [f64; 3],
    pub max: [f64; 3],
}

impl Aabb {
    pub fn new(min: [f64; 3], max: [f64; 3]) -> Self { Self { min, max } }

    pub fn from_center_half(center: [f64; 3], half: [f64; 3]) -> Self {
        Self {
            min: [center[0] - half[0], center[1] - half[1], center[2] - half[2]],
            max: [center[0] + half[0], center[1] + half[1], center[2] + half[2]],
        }
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        self.min[0] <= other.max[0] && self.max[0] >= other.min[0]
            && self.min[1] <= other.max[1] && self.max[1] >= other.min[1]
            && self.min[2] <= other.max[2] && self.max[2] >= other.min[2]
    }

    pub fn contains_point(&self, p: [f64; 3]) -> bool {
        p[0] >= self.min[0] && p[0] <= self.max[0]
            && p[1] >= self.min[1] && p[1] <= self.max[1]
            && p[2] >= self.min[2] && p[2] <= self.max[2]
    }

    pub fn surface_area(&self) -> f64 {
        let d = [
            self.max[0] - self.min[0],
            self.max[1] - self.min[1],
            self.max[2] - self.min[2],
        ];
        2.0 * (d[0] * d[1] + d[1] * d[2] + d[2] * d[0])
    }

    pub fn union(&self, other: &Aabb) -> Aabb {
        Aabb {
            min: [
                self.min[0].min(other.min[0]),
                self.min[1].min(other.min[1]),
                self.min[2].min(other.min[2]),
            ],
            max: [
                self.max[0].max(other.max[0]),
                self.max[1].max(other.max[1]),
                self.max[2].max(other.max[2]),
            ],
        }
    }

    pub fn centroid(&self) -> [f64; 3] {
        [
            (self.min[0] + self.max[0]) * 0.5,
            (self.min[1] + self.max[1]) * 0.5,
            (self.min[2] + self.max[2]) * 0.5,
        ]
    }

    pub fn penetration_depth(&self, other: &Aabb) -> Option<([f64; 3], f64)> {
        if !self.intersects(other) { return None; }
        let overlaps = [
            (self.max[0] - other.min[0]).min(other.max[0] - self.min[0]),
            (self.max[1] - other.min[1]).min(other.max[1] - self.min[1]),
            (self.max[2] - other.min[2]).min(other.max[2] - self.min[2]),
        ];
        let (axis, depth) = overlaps.iter().copied().enumerate().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        let mut normal = [0.0f64; 3];
        normal[axis] = if self.centroid()[axis] < other.centroid()[axis] { -1.0 } else { 1.0 };
        Some((normal, depth))
    }
}

enum BvhNode {
    Leaf { aabb: Aabb, object_id: ObjectId },
    Internal { aabb: Aabb, left: usize, right: usize },
}

pub struct Bvh {
    nodes: Vec<BvhNode>,
}

impl Bvh {
    pub fn new() -> Self { Self { nodes: Vec::new() } }

    pub fn build(&mut self, entries: &[(ObjectId, Aabb)]) {
        self.nodes.clear();
        if entries.is_empty() { return; }
        let mut indices: Vec<usize> = (0..entries.len()).collect();
        self.build_recursive(entries, &mut indices);
    }

    fn build_recursive(&mut self, entries: &[(ObjectId, Aabb)], indices: &mut [usize]) -> usize {
        if indices.len() == 1 {
            let (id, aabb) = entries[indices[0]].clone();
            let idx = self.nodes.len();
            self.nodes.push(BvhNode::Leaf { aabb, object_id: id });
            return idx;
        }
        let mut combined = entries[indices[0]].1.clone();
        for &i in indices.iter().skip(1) {
            combined = combined.union(&entries[i].1);
        }
        let extent = [
            combined.max[0] - combined.min[0],
            combined.max[1] - combined.min[1],
            combined.max[2] - combined.min[2],
        ];
        let axis = if extent[0] >= extent[1] && extent[0] >= extent[2] { 0 }
            else if extent[1] >= extent[2] { 1 } else { 2 };
        indices.sort_by(|&a, &b| {
            let ca = entries[a].1.centroid()[axis];
            let cb = entries[b].1.centroid()[axis];
            ca.partial_cmp(&cb).unwrap()
        });
        let mid = indices.len() / 2;
        let (left_idx, right_idx) = indices.split_at_mut(mid);
        let left = self.build_recursive(entries, left_idx);
        let right = self.build_recursive(entries, right_idx);
        let node_idx = self.nodes.len();
        self.nodes.push(BvhNode::Internal { aabb: combined, left, right });
        node_idx
    }

    pub fn query_overlaps(&self, query: &Aabb) -> Vec<ObjectId> {
        let mut result = Vec::new();
        if self.nodes.is_empty() { return result; }
        let root = self.nodes.len() - 1;
        self.query_recursive(root, query, &mut result);
        result
    }

    fn query_recursive(&self, idx: usize, query: &Aabb, out: &mut Vec<ObjectId>) {
        match &self.nodes[idx] {
            BvhNode::Leaf { aabb, object_id } => {
                if aabb.intersects(query) { out.push(*object_id); }
            }
            BvhNode::Internal { aabb, left, right } => {
                if aabb.intersects(query) {
                    self.query_recursive(*left, query, out);
                    self.query_recursive(*right, query, out);
                }
            }
        }
    }
}

impl Default for Bvh {
    fn default() -> Self { Self::new() }
}

#[derive(Clone, Debug)]
pub struct CollisionPair {
    pub a: ObjectId,
    pub b: ObjectId,
    pub normal: [f64; 3],
    pub depth: f64,
    pub is_trigger: bool,
}

pub fn collect_collision_pairs(entries: &[(ObjectId, Aabb, bool)]) -> Vec<CollisionPair> {
    let mut pairs = Vec::new();
    for i in 0..entries.len() {
        for j in (i + 1)..entries.len() {
            let (id_a, aabb_a, trig_a) = &entries[i];
            let (id_b, aabb_b, trig_b) = &entries[j];
            if let Some((normal, depth)) = aabb_a.penetration_depth(aabb_b) {
                pairs.push(CollisionPair {
                    a: *id_a,
                    b: *id_b,
                    normal,
                    depth,
                    is_trigger: *trig_a || *trig_b,
                });
            }
        }
    }
    pairs
}

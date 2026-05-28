use crate::core::engine::rendering::raytracing::Vec3;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub struct SnapshotState {
    pub timestamp: f64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: [f64; 4],
}

impl SnapshotState {
    pub fn lerp(&self, other: &Self, t: f64) -> Self {
        let lerp_v = |a: Vec3, b: Vec3| -> Vec3 { a + (b - a) * t };
        let lerp_q = |a: [f64; 4], b: [f64; 4]| -> [f64; 4] {
            let dot = a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3];
            let b_adj = if dot < 0.0 { [-b[0], -b[1], -b[2], -b[3]] } else { b };
            let q = [
                a[0] + (b_adj[0] - a[0]) * t,
                a[1] + (b_adj[1] - a[1]) * t,
                a[2] + (b_adj[2] - a[2]) * t,
                a[3] + (b_adj[3] - a[3]) * t,
            ];
            let len = (q[0]*q[0] + q[1]*q[1] + q[2]*q[2] + q[3]*q[3]).sqrt().max(f64::EPSILON);
            [q[0]/len, q[1]/len, q[2]/len, q[3]/len]
        };
        Self {
            timestamp: self.timestamp + (other.timestamp - self.timestamp) * t,
            position: lerp_v(self.position, other.position),
            velocity: lerp_v(self.velocity, other.velocity),
            rotation: lerp_q(self.rotation, other.rotation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct InterpolatedEntity {
    pub snapshots: Vec<SnapshotState>,
    pub interpolation_delay: f64,
    pub max_snapshots: usize,
}

impl InterpolatedEntity {
    pub fn new(interpolation_delay: f64, max_snapshots: usize) -> Self {
        Self { snapshots: Vec::new(), interpolation_delay, max_snapshots }
    }

    pub fn push_snapshot(&mut self, state: SnapshotState) {
        self.snapshots.push(state);
        self.snapshots.sort_by(|a, b| a.timestamp.partial_cmp(&b.timestamp).unwrap());
        while self.snapshots.len() > self.max_snapshots {
            self.snapshots.remove(0);
        }
    }

    pub fn sample(&self, current_time: f64) -> Option<SnapshotState> {
        let render_time = current_time - self.interpolation_delay;
        if self.snapshots.is_empty() { return None; }
        if render_time <= self.snapshots[0].timestamp {
            return Some(self.snapshots[0]);
        }
        let last = self.snapshots.last().unwrap();
        if render_time >= last.timestamp {
            return Some(*last);
        }
        for i in 0..self.snapshots.len() - 1 {
            let a = &self.snapshots[i];
            let b = &self.snapshots[i + 1];
            if render_time >= a.timestamp && render_time <= b.timestamp {
                let span = b.timestamp - a.timestamp;
                let t = if span > f64::EPSILON { (render_time - a.timestamp) / span } else { 0.0 };
                return Some(a.lerp(b, t));
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub struct LagCompensator {
    pub entity_histories: HashMap<u64, InterpolatedEntity>,
    pub interpolation_delay: f64,
    pub max_snapshots: usize,
}

impl LagCompensator {
    pub fn new(interpolation_delay: f64, max_snapshots: usize) -> Self {
        Self {
            entity_histories: HashMap::new(),
            interpolation_delay,
            max_snapshots,
        }
    }

    pub fn record(&mut self, entity_id: u64, state: SnapshotState) {
        let entity = self.entity_histories
            .entry(entity_id)
            .or_insert_with(|| InterpolatedEntity::new(self.interpolation_delay, self.max_snapshots));
        entity.push_snapshot(state);
    }

    pub fn rewind(&self, entity_id: u64, to_time: f64) -> Option<SnapshotState> {
        let entity = self.entity_histories.get(&entity_id)?;
        let render_time = to_time;
        if entity.snapshots.is_empty() { return None; }
        for i in 0..entity.snapshots.len() - 1 {
            let a = &entity.snapshots[i];
            let b = &entity.snapshots[i + 1];
            if render_time >= a.timestamp && render_time <= b.timestamp {
                let span = b.timestamp - a.timestamp;
                let t = if span > f64::EPSILON { (render_time - a.timestamp) / span } else { 0.0 };
                return Some(a.lerp(b, t));
            }
        }
        entity.snapshots.iter().min_by(|a, b| {
            (a.timestamp - to_time).abs().partial_cmp(&(b.timestamp - to_time).abs()).unwrap()
        }).copied()
    }

    pub fn tracked_entity_count(&self) -> usize {
        self.entity_histories.len()
    }
}

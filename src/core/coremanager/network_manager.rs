use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::scene::graph::SceneGraph;

#[derive(Debug, Clone, Copy)]
pub struct NetworkSnapshot {
    pub frame_index: u64,
    pub scene_origin: Vec3,
    pub scene_radius: f64,
    pub node_count: usize,
    pub checksum: u64,
}

#[derive(Debug, Clone, Copy)]
pub struct NetworkStatus {
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub last_checksum: u64,
}

#[derive(Debug, Clone)]
pub struct NetworkManager {
    remote_clients: usize,
    status: NetworkStatus,
}

impl NetworkManager {
    pub fn new(remote_clients: usize) -> Self {
        Self {
            remote_clients: remote_clients.max(1),
            status: NetworkStatus {
                latency_ms: 8.0,
                packet_loss: 0.0,
                last_checksum: 0,
            },
        }
    }

    pub fn sync_scene(&mut self, graph: &SceneGraph, frame_index: u64) -> NetworkSnapshot {
        let checksum = frame_index
            ^ ((graph.scene_radius() * 1000.0) as u64)
            ^ (graph.node_count() as u64 * 31)
            ^ (graph.luminous_node_count() as u64 * 131);

        self.status.latency_ms = (6.0 + graph.scene_radius() * 0.35).clamp(4.0, 24.0);
        self.status.packet_loss = (0.003 * self.remote_clients as f64).clamp(0.0, 0.05);
        self.status.last_checksum = checksum;

        NetworkSnapshot {
            frame_index,
            scene_origin: graph.focus_point(),
            scene_radius: graph.scene_radius(),
            node_count: graph.node_count(),
            checksum,
        }
    }

    pub fn status(&self) -> NetworkStatus {
        self.status
    }

    pub fn remote_client_count(&self) -> usize {
        self.remote_clients
    }
}

// ── Render sync server ──────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct RenderSyncServer {
    capacity: usize,
}

impl RenderSyncServer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
        }
    }

    pub fn publish(&self, _frame_index: u64, _snapshot: &NetworkSnapshot) -> usize {
        self.capacity
    }

    pub fn client_count(&self) -> usize {
        self.capacity
    }
}

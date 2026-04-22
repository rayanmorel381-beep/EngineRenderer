use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::scene::graph::SceneGraph;

/// Per-frame network snapshot derived from scene state.
#[derive(Debug, Clone, Copy)]
pub struct NetworkSnapshot {
    /// Frame index associated with this snapshot.
    pub frame_index: u64,
    /// Estimated scene origin.
    pub scene_origin: Vec3,
    /// Estimated scene radius.
    pub scene_radius: f64,
    /// Number of nodes in the synchronized scene.
    pub node_count: usize,
    /// Lightweight checksum used for sync diagnostics.
    pub checksum: u64,
}

/// Current synthetic network status indicators.
#[derive(Debug, Clone, Copy)]
pub struct NetworkStatus {
    /// Estimated end-to-end latency in milliseconds.
    pub latency_ms: f64,
    /// Estimated packet loss ratio.
    pub packet_loss: f64,
    /// Last published checksum.
    pub last_checksum: u64,
}

/// Manages network synchronization signals for scene updates.
#[derive(Debug, Clone)]
pub struct NetworkManager {
    remote_clients: usize,
    status: NetworkStatus,
}

impl NetworkManager {
    /// Creates a network manager for a given number of remote clients.
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

    /// Synchronizes scene data and returns the produced snapshot.
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

    /// Returns the current network status.
    pub fn status(&self) -> NetworkStatus {
        self.status
    }

    /// Returns the configured number of remote clients.
    pub fn remote_client_count(&self) -> usize {
        self.remote_clients
    }
}

// ── Render sync server ──────────────────────────────────────────────────

/// Lightweight publish-side sync server abstraction.
#[derive(Debug, Clone, Copy)]
pub struct RenderSyncServer {
    capacity: usize,
}

impl RenderSyncServer {
    /// Creates a sync server with a maximum client capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
        }
    }

    /// Publishes one snapshot and returns delivered client count.
    pub fn publish(&self, frame_index: u64, snapshot: &NetworkSnapshot) -> usize {
        let scene_scale = snapshot.node_count.max(1).saturating_div(128).max(1);
        let frame_scale = ((frame_index ^ snapshot.frame_index) as usize & 0x3).saturating_add(1);
        self.capacity.min(scene_scale.saturating_mul(frame_scale)).max(1)
    }

    /// Returns maximum supported client count.
    pub fn client_count(&self) -> usize {
        self.capacity
    }
}

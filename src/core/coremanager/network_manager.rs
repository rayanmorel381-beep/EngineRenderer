use crate::core::engine::rendering::raytracing::Vec3;
use crate::core::engine::scene::graph::SceneGraph;

/// Snapshot réseau d'une scène synchronisée pour une frame.
#[derive(Debug, Clone, Copy)]
pub struct NetworkSnapshot {
    pub frame_index: u64,
    pub scene_origin: Vec3,
    pub scene_radius: f64,
    pub node_count: usize,
    pub checksum: u64,
}

/// État réseau synthétique observé par le moteur.
#[derive(Debug, Clone, Copy)]
pub struct NetworkStatus {
    pub latency_ms: f64,
    pub packet_loss: f64,
    pub last_checksum: u64,
}

/// Gestionnaire de synchronisation réseau simplifié.
#[derive(Debug, Clone)]
pub struct NetworkManager {
    remote_clients: usize,
    status: NetworkStatus,
}

impl NetworkManager {
    /// Crée un gestionnaire réseau avec un nombre de clients distants attendu.
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

    /// Synchronise un graphe de scène et retourne le snapshot diffusé.
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

    /// Retourne l'état réseau courant.
    pub fn status(&self) -> NetworkStatus {
        self.status
    }

    /// Retourne le nombre de clients distants configuré.
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
    /// Crée un serveur de synchro rendu avec une capacité client max.
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
        }
    }

    /// Publie un snapshot et retourne le nombre de clients effectivement servis.
    pub fn publish(&self, frame_index: u64, snapshot: &NetworkSnapshot) -> usize {
        let scene_scale = snapshot.node_count.max(1).saturating_div(128).max(1);
        let frame_scale = ((frame_index ^ snapshot.frame_index) as usize & 0x3).saturating_add(1);
        self.capacity.min(scene_scale.saturating_mul(frame_scale)).max(1)
    }

    /// Retourne le nombre de clients pouvant être servis.
    pub fn client_count(&self) -> usize {
        self.capacity
    }
}

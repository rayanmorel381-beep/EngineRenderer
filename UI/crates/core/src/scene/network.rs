use crate::scene::ObjectId;

#[derive(Clone, Debug, PartialEq)]
pub enum NetworkRole {
    Authority,
    SimulatedProxy,
    AutonomousProxy,
    None,
}

impl NetworkRole {
    pub fn label(&self) -> &'static str {
        match self { Self::Authority => "Authority", Self::SimulatedProxy => "Simulated Proxy", Self::AutonomousProxy => "Autonomous Proxy", Self::None => "Aucun" }
    }
    pub const ALL: [NetworkRole; 4] = [NetworkRole::Authority, NetworkRole::SimulatedProxy, NetworkRole::AutonomousProxy, NetworkRole::None];
}

impl Default for NetworkRole { fn default() -> Self { Self::Authority } }

#[derive(Clone, Debug)]
pub struct ReplicationConfig {
    pub enabled: bool,
    pub role: NetworkRole,
    pub replicate_position: bool,
    pub replicate_rotation: bool,
    pub replicate_scale: bool,
    pub replicate_physics: bool,
    pub net_update_rate: f64,
    pub net_priority: f64,
    pub relevancy_radius: f64,
}

impl Default for ReplicationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            role: NetworkRole::Authority,
            replicate_position: true,
            replicate_rotation: true,
            replicate_scale: false,
            replicate_physics: false,
            net_update_rate: 30.0,
            net_priority: 1.0,
            relevancy_radius: 100.0,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct NetworkState {
    pub tick: u64,
    pub role: NetworkRole,
    pub ping_ms: f64,
    pub packet_loss: f64,
    pub max_clients: usize,
    pub server_fps: f64,
    pub dedicated: bool,
}

impl NetworkState {
    pub fn new() -> Self {
        Self { max_clients: 64, server_fps: 64.0, ..Self::default() }
    }
}

#[derive(Clone, Debug)]
pub struct NetworkObject {
    pub object_id: ObjectId,
    pub net_id: u32,
    pub config: ReplicationConfig,
    pub owner_connection: Option<u32>,
}

impl NetworkObject {
    pub fn new(object_id: ObjectId, net_id: u32) -> Self {
        Self { object_id, net_id, config: ReplicationConfig::default(), owner_connection: None }
    }
}

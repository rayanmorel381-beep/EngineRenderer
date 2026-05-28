use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum VfxPortKind {
    Float,
    Vec3,
    Color,
    Int,
    Bool,
}

impl VfxPortKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Float => "Float",
            Self::Vec3 => "Vec3",
            Self::Color => "Color",
            Self::Int => "Int",
            Self::Bool => "Bool",
        }
    }
}

#[derive(Clone, Debug)]
pub struct VfxPort {
    pub id: u32,
    pub name: String,
    pub kind: VfxPortKind,
    pub is_output: bool,
    pub connected_to: Option<(u32, u32)>,
}

impl VfxPort {
    pub fn input(id: u32, name: impl Into<String>, kind: VfxPortKind) -> Self {
        Self { id, name: name.into(), kind, is_output: false, connected_to: None }
    }
    pub fn output(id: u32, name: impl Into<String>, kind: VfxPortKind) -> Self {
        Self { id, name: name.into(), kind, is_output: true, connected_to: None }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum VfxNodeCategory {
    Emitter,
    Update,
    Render,
    Event,
    Math,
    Noise,
    Force,
}

impl VfxNodeCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Emitter => "Émetteur",
            Self::Update => "Mise à jour",
            Self::Render => "Rendu",
            Self::Event => "Événement",
            Self::Math => "Math",
            Self::Noise => "Bruit",
            Self::Force => "Force",
        }
    }
    pub const ALL: [VfxNodeCategory; 7] = [
        VfxNodeCategory::Emitter, VfxNodeCategory::Update, VfxNodeCategory::Render,
        VfxNodeCategory::Event, VfxNodeCategory::Math, VfxNodeCategory::Noise, VfxNodeCategory::Force,
    ];
}

#[derive(Clone, Debug)]
pub struct VfxNode {
    pub id: u32,
    pub title: String,
    pub category: VfxNodeCategory,
    pub inputs: Vec<VfxPort>,
    pub outputs: Vec<VfxPort>,
    pub position: [f64; 2],
    pub value_float: f64,
    pub value_color: [f64; 4],
    pub value_vec3: [f64; 3],
    pub collapsed: bool,
}

impl VfxNode {
    pub fn spawn_rate(id: u32) -> Self {
        Self {
            id, title: "Spawn Rate".into(), category: VfxNodeCategory::Emitter,
            inputs: vec![VfxPort::input(0, "Rate", VfxPortKind::Float)],
            outputs: vec![VfxPort::output(1, "Particles", VfxPortKind::Int)],
            position: [0.0, 0.0], value_float: 10.0, value_color: [1.0; 4], value_vec3: [0.0; 3], collapsed: false,
        }
    }
    pub fn initial_velocity(id: u32) -> Self {
        Self {
            id, title: "Initial Velocity".into(), category: VfxNodeCategory::Emitter,
            inputs: vec![VfxPort::input(0, "Velocity", VfxPortKind::Vec3)],
            outputs: vec![VfxPort::output(1, "Out", VfxPortKind::Vec3)],
            position: [200.0, 0.0], value_float: 0.0, value_color: [1.0; 4], value_vec3: [0.0, 5.0, 0.0], collapsed: false,
        }
    }
    pub fn color_over_lifetime(id: u32) -> Self {
        Self {
            id, title: "Color over Lifetime".into(), category: VfxNodeCategory::Update,
            inputs: vec![VfxPort::input(0, "Start", VfxPortKind::Color), VfxPort::input(1, "End", VfxPortKind::Color)],
            outputs: vec![VfxPort::output(2, "Color", VfxPortKind::Color)],
            position: [0.0, 120.0], value_float: 0.0, value_color: [1.0, 0.5, 0.0, 1.0], value_vec3: [0.0; 3], collapsed: false,
        }
    }
    pub fn turbulence(id: u32) -> Self {
        Self {
            id, title: "Turbulence".into(), category: VfxNodeCategory::Force,
            inputs: vec![VfxPort::input(0, "Intensity", VfxPortKind::Float), VfxPort::input(1, "Frequency", VfxPortKind::Float)],
            outputs: vec![VfxPort::output(2, "Force", VfxPortKind::Vec3)],
            position: [200.0, 120.0], value_float: 1.0, value_color: [1.0; 4], value_vec3: [0.0; 3], collapsed: false,
        }
    }
    pub fn sprite_renderer(id: u32) -> Self {
        Self {
            id, title: "Sprite Renderer".into(), category: VfxNodeCategory::Render,
            inputs: vec![VfxPort::input(0, "Color", VfxPortKind::Color), VfxPort::input(1, "Size", VfxPortKind::Float)],
            outputs: vec![],
            position: [400.0, 60.0], value_float: 0.1, value_color: [1.0; 4], value_vec3: [0.0; 3], collapsed: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct VfxConnection {
    pub from_node: u32,
    pub from_port: u32,
    pub to_node: u32,
    pub to_port: u32,
}

#[derive(Clone, Debug)]
pub struct VfxGraph {
    pub name: String,
    pub nodes: Vec<VfxNode>,
    pub connections: Vec<VfxConnection>,
    pub max_particles: usize,
    pub loop_duration: f64,
    pub looping: bool,
    pub enabled: bool,
    next_id: u32,
}

impl Default for VfxGraph {
    fn default() -> Self {
        let mut g = Self {
            name: "VFX Graph".to_string(),
            nodes: Vec::new(),
            connections: Vec::new(),
            max_particles: 1000,
            loop_duration: 2.0,
            looping: true,
            enabled: true,
            next_id: 0,
        };
        let n0 = VfxNode::spawn_rate(g.alloc_id());
        let n1 = VfxNode::initial_velocity(g.alloc_id());
        let n2 = VfxNode::color_over_lifetime(g.alloc_id());
        let n3 = VfxNode::sprite_renderer(g.alloc_id());
        g.nodes.push(n0);
        g.nodes.push(n1);
        g.nodes.push(n2);
        g.nodes.push(n3);
        g
    }
}

impl VfxGraph {
    pub fn new() -> Self { Self::default() }

    fn alloc_id(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    pub fn add_node(&mut self, mut node: VfxNode) -> u32 {
        let id = self.alloc_id();
        node.id = id;
        self.nodes.push(node);
        id
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.retain(|n| n.id != id);
        self.connections.retain(|c| c.from_node != id && c.to_node != id);
    }

    pub fn connect(&mut self, from_node: u32, from_port: u32, to_node: u32, to_port: u32) {
        self.connections.retain(|c| !(c.to_node == to_node && c.to_port == to_port));
        self.connections.push(VfxConnection { from_node, from_port, to_node, to_port });
    }

    pub fn disconnect(&mut self, to_node: u32, to_port: u32) {
        self.connections.retain(|c| !(c.to_node == to_node && c.to_port == to_port));
    }

    pub fn node_params(&self) -> HashMap<String, f64> {
        let mut map = HashMap::new();
        for node in &self.nodes {
            map.insert(node.title.clone(), node.value_float);
        }
        map
    }
}

use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum PcgNodeKind {
    PointScatter,
    Filter,
    Transform,
    Noise,
    MeshSpawn,
    Select,
    Copy,
    Debug,
}

impl PcgNodeKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PointScatter => "Scatter points",
            Self::Filter => "Filtrer",
            Self::Transform => "Transformer",
            Self::Noise => "Bruit",
            Self::MeshSpawn => "Spawn mesh",
            Self::Select => "Sélectionner",
            Self::Copy => "Copier",
            Self::Debug => "Déboguer",
        }
    }
    pub const ALL: [PcgNodeKind; 8] = [
        PcgNodeKind::PointScatter, PcgNodeKind::Filter, PcgNodeKind::Transform, PcgNodeKind::Noise,
        PcgNodeKind::MeshSpawn, PcgNodeKind::Select, PcgNodeKind::Copy, PcgNodeKind::Debug,
    ];
}

#[derive(Clone, Debug)]
pub struct PcgNode {
    pub id: u32,
    pub kind: PcgNodeKind,
    pub position: [f64; 2],
    pub params: HashMap<String, f64>,
    pub param_strings: HashMap<String, String>,
    pub enabled: bool,
    pub seed: u64,
    pub output_count: usize,
}

impl PcgNode {
    pub fn scatter(id: u32) -> Self {
        let mut params = HashMap::new();
        params.insert("count".to_string(), 100.0);
        params.insert("radius".to_string(), 10.0);
        Self { id, kind: PcgNodeKind::PointScatter, position: [0.0, 0.0], params, param_strings: HashMap::new(), enabled: true, seed: 0, output_count: 0 }
    }
    pub fn filter(id: u32) -> Self {
        let mut params = HashMap::new();
        params.insert("min_slope".to_string(), 0.0);
        params.insert("max_slope".to_string(), 45.0);
        params.insert("min_height".to_string(), -100.0);
        params.insert("max_height".to_string(), 100.0);
        Self { id, kind: PcgNodeKind::Filter, position: [200.0, 0.0], params, param_strings: HashMap::new(), enabled: true, seed: 0, output_count: 0 }
    }
    pub fn mesh_spawn(id: u32) -> Self {
        let mut params = HashMap::new();
        params.insert("scale_min".to_string(), 0.8);
        params.insert("scale_max".to_string(), 1.2);
        let mut strs = HashMap::new();
        strs.insert("mesh".to_string(), String::new());
        Self { id, kind: PcgNodeKind::MeshSpawn, position: [400.0, 0.0], params, param_strings: strs, enabled: true, seed: 0, output_count: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct PcgEdge {
    pub from: u32,
    pub to: u32,
}

#[derive(Clone, Debug)]
pub struct PcgGraph {
    pub name: String,
    pub nodes: Vec<PcgNode>,
    pub edges: Vec<PcgEdge>,
    pub enabled: bool,
    pub auto_execute: bool,
    next_id: u32,
}

impl Default for PcgGraph {
    fn default() -> Self {
        let mut g = Self { name: "PCG Graph".to_string(), nodes: Vec::new(), edges: Vec::new(), enabled: true, auto_execute: true, next_id: 0 };
        let n0 = PcgNode::scatter(g.alloc_id());
        let n1 = PcgNode::filter(g.alloc_id());
        let n2 = PcgNode::mesh_spawn(g.alloc_id());
        let e0 = PcgEdge { from: n0.id, to: n1.id };
        let e1 = PcgEdge { from: n1.id, to: n2.id };
        g.nodes.push(n0);
        g.nodes.push(n1);
        g.nodes.push(n2);
        g.edges.push(e0);
        g.edges.push(e1);
        g
    }
}

impl PcgGraph {
    pub fn new() -> Self { Self::default() }

    fn alloc_id(&mut self) -> u32 { let id = self.next_id; self.next_id += 1; id }

    pub fn add_node(&mut self, kind: PcgNodeKind) -> u32 {
        let id = self.alloc_id();
        let node = PcgNode { id, kind, position: [0.0, 0.0], params: HashMap::new(), param_strings: HashMap::new(), enabled: true, seed: 0, output_count: 0 };
        self.nodes.push(node);
        id
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.retain(|n| n.id != id);
        self.edges.retain(|e| e.from != id && e.to != id);
    }

    pub fn connect(&mut self, from: u32, to: u32) {
        self.edges.retain(|e| e.to != to);
        self.edges.push(PcgEdge { from, to });
    }
}

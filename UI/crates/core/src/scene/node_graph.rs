#[derive(Clone, Debug, PartialEq)]
pub enum PinKind { Exec, Bool, Int, Float, Vec3, String, Object }

impl PinKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Exec => "►", Self::Bool => "bool", Self::Int => "int",
            Self::Float => "f64", Self::Vec3 => "vec3", Self::String => "str", Self::Object => "obj",
        }
    }
    pub fn color(&self) -> [f64; 4] {
        match self {
            Self::Exec => [1.0, 1.0, 1.0, 1.0], Self::Bool => [0.8, 0.2, 0.2, 1.0],
            Self::Int => [0.2, 0.7, 0.2, 1.0], Self::Float => [0.0, 0.6, 0.9, 1.0],
            Self::Vec3 => [0.9, 0.5, 0.0, 1.0], Self::String => [0.8, 0.4, 0.9, 1.0],
            Self::Object => [0.2, 0.8, 0.6, 1.0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodePin {
    pub id: usize,
    pub label: String,
    pub kind: PinKind,
    pub is_output: bool,
    pub connected_to: Vec<(usize, usize)>,
}

impl NodePin {
    pub fn input(id: usize, label: &str, kind: PinKind) -> Self {
        Self { id, label: label.into(), kind, is_output: false, connected_to: Vec::new() }
    }
    pub fn output(id: usize, label: &str, kind: PinKind) -> Self {
        Self { id, label: label.into(), kind, is_output: true, connected_to: Vec::new() }
    }
}

#[derive(Clone, Debug)]
pub struct ScriptNode {
    pub id: usize,
    pub title: String,
    pub category: String,
    pub inputs: Vec<NodePin>,
    pub outputs: Vec<NodePin>,
    pub position: [f64; 2],
    pub comment: String,
    pub collapsed: bool,
    pub value_float: f64,
    pub value_string: String,
}

impl ScriptNode {
    pub fn event_tick(id: usize) -> Self {
        Self {
            id, title: "Event Tick".into(), category: "Events".into(),
            inputs: Vec::new(),
            outputs: vec![NodePin::output(0, "Exec", PinKind::Exec), NodePin::output(1, "DeltaTime", PinKind::Float)],
            position: [50.0, 100.0], comment: String::new(), collapsed: false, value_float: 0.0, value_string: String::new(),
        }
    }
    pub fn event_begin_play(id: usize) -> Self {
        Self {
            id, title: "Event BeginPlay".into(), category: "Events".into(),
            inputs: Vec::new(),
            outputs: vec![NodePin::output(0, "Exec", PinKind::Exec)],
            position: [50.0, 200.0], comment: String::new(), collapsed: false, value_float: 0.0, value_string: String::new(),
        }
    }
    pub fn float_add(id: usize) -> Self {
        Self {
            id, title: "Float + Float".into(), category: "Math".into(),
            inputs: vec![NodePin::input(0, "A", PinKind::Float), NodePin::input(1, "B", PinKind::Float)],
            outputs: vec![NodePin::output(0, "Result", PinKind::Float)],
            position: [300.0, 100.0], comment: String::new(), collapsed: false, value_float: 0.0, value_string: String::new(),
        }
    }
    pub fn set_position(id: usize) -> Self {
        Self {
            id, title: "Set Position".into(), category: "Transform".into(),
            inputs: vec![NodePin::input(0, "Exec", PinKind::Exec), NodePin::input(1, "Target", PinKind::Object), NodePin::input(2, "Position", PinKind::Vec3)],
            outputs: vec![NodePin::output(0, "Exec", PinKind::Exec)],
            position: [500.0, 100.0], comment: String::new(), collapsed: false, value_float: 0.0, value_string: String::new(),
        }
    }
    pub fn branch(id: usize) -> Self {
        Self {
            id, title: "Branch".into(), category: "Flow".into(),
            inputs: vec![NodePin::input(0, "Exec", PinKind::Exec), NodePin::input(1, "Condition", PinKind::Bool)],
            outputs: vec![NodePin::output(0, "True", PinKind::Exec), NodePin::output(1, "False", PinKind::Exec)],
            position: [300.0, 200.0], comment: String::new(), collapsed: false, value_float: 0.0, value_string: String::new(),
        }
    }
    pub fn print_string(id: usize) -> Self {
        Self {
            id, title: "Print String".into(), category: "Debug".into(),
            inputs: vec![NodePin::input(0, "Exec", PinKind::Exec), NodePin::input(1, "String", PinKind::String)],
            outputs: vec![NodePin::output(0, "Exec", PinKind::Exec)],
            position: [500.0, 200.0], comment: String::new(), collapsed: false, value_float: 0.0, value_string: "Hello".into(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct NodeConnection {
    pub from_node: usize,
    pub from_pin: usize,
    pub to_node: usize,
    pub to_pin: usize,
}

#[derive(Clone, Debug)]
pub struct NodeGraph {
    pub name: String,
    pub nodes: Vec<ScriptNode>,
    pub connections: Vec<NodeConnection>,
    pub next_id: usize,
}

impl NodeGraph {
    pub fn new(name: impl Into<String>) -> Self {
        let mut g = Self { name: name.into(), nodes: Vec::new(), connections: Vec::new(), next_id: 0 };
        let id0 = g.next_id; g.next_id += 1;
        let id1 = g.next_id; g.next_id += 1;
        g.nodes.push(ScriptNode::event_tick(id0));
        g.nodes.push(ScriptNode::event_begin_play(id1));
        g
    }

    pub fn add_node(&mut self, node: ScriptNode) -> usize {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    pub fn connect(&mut self, from_node: usize, from_pin: usize, to_node: usize, to_pin: usize) {
        self.connections.push(NodeConnection { from_node, from_pin, to_node, to_pin });
    }

    pub fn disconnect(&mut self, from_node: usize, from_pin: usize, to_node: usize, to_pin: usize) {
        self.connections.retain(|c| !(c.from_node == from_node && c.from_pin == from_pin && c.to_node == to_node && c.to_pin == to_pin));
    }

    pub fn remove_node(&mut self, id: usize) {
        self.nodes.retain(|n| n.id != id);
        self.connections.retain(|c| c.from_node != id && c.to_node != id);
    }
}

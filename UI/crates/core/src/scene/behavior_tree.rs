use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub enum BtStatus {
    Success,
    Failure,
    Running,
}

#[derive(Clone, Debug)]
pub enum BlackboardValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Vec3([f64; 3]),
}

#[derive(Clone, Debug, Default)]
pub struct Blackboard {
    pub entries: HashMap<String, BlackboardValue>,
}

impl Blackboard {
    pub fn new() -> Self { Self::default() }
    pub fn set_bool(&mut self, key: &str, v: bool) { self.entries.insert(key.into(), BlackboardValue::Bool(v)); }
    pub fn set_float(&mut self, key: &str, v: f64) { self.entries.insert(key.into(), BlackboardValue::Float(v)); }
    pub fn set_int(&mut self, key: &str, v: i64) { self.entries.insert(key.into(), BlackboardValue::Int(v)); }
    pub fn set_string(&mut self, key: &str, v: &str) { self.entries.insert(key.into(), BlackboardValue::String(v.into())); }
    pub fn set_vec3(&mut self, key: &str, v: [f64; 3]) { self.entries.insert(key.into(), BlackboardValue::Vec3(v)); }
    pub fn get_bool(&self, key: &str) -> Option<bool> { match self.entries.get(key) { Some(BlackboardValue::Bool(v)) => Some(*v), _ => None } }
    pub fn get_float(&self, key: &str) -> Option<f64> { match self.entries.get(key) { Some(BlackboardValue::Float(v)) => Some(*v), _ => None } }
    pub fn get_vec3(&self, key: &str) -> Option<[f64; 3]> { match self.entries.get(key) { Some(BlackboardValue::Vec3(v)) => Some(*v), _ => None } }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BtNodeKind {
    Sequence,
    Selector,
    Parallel,
    Inverter,
    Repeater { times: usize },
    Leaf { action: String },
    Condition { key: String, op: ConditionOp, value_float: f64 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConditionOp { Greater, Less, Equal, NotEqual }

impl ConditionOp {
    pub fn label(&self) -> &'static str {
        match self { Self::Greater => ">", Self::Less => "<", Self::Equal => "==", Self::NotEqual => "!=" }
    }
    pub const ALL: [ConditionOp; 4] = [ConditionOp::Greater, ConditionOp::Less, ConditionOp::Equal, ConditionOp::NotEqual];
}

#[derive(Clone, Debug)]
pub struct BtNode {
    pub id: usize,
    pub label: String,
    pub kind: BtNodeKind,
    pub children: Vec<usize>,
    pub repeat_count: usize,
}

impl BtNode {
    pub fn new(id: usize, kind: BtNodeKind) -> Self {
        let label = match &kind {
            BtNodeKind::Sequence => "Sequence".to_owned(),
            BtNodeKind::Selector => "Selector".to_owned(),
            BtNodeKind::Parallel => "Parallel".to_owned(),
            BtNodeKind::Inverter => "Inverter".to_owned(),
            BtNodeKind::Repeater { times } => format!("Repeat×{times}"),
            BtNodeKind::Leaf { action } => action.clone(),
            BtNodeKind::Condition { key, op, value_float } => format!("{key} {} {value_float}", op.label()),
        };
        Self { id, label, kind, children: Vec::new(), repeat_count: 0 }
    }
}

#[derive(Clone, Debug)]
pub struct BehaviorTree {
    pub nodes: Vec<BtNode>,
    pub root: Option<usize>,
    pub blackboard: Blackboard,
    pub next_id: usize,
}

impl Default for BehaviorTree {
    fn default() -> Self {
        let mut bt = Self { nodes: Vec::new(), root: None, blackboard: Blackboard::new(), next_id: 0 };
        let root_id = bt.add_node(BtNodeKind::Selector);
        bt.root = Some(root_id);
        let patrol = bt.add_node(BtNodeKind::Sequence);
        let chase = bt.add_node(BtNodeKind::Sequence);
        bt.nodes[root_id].children = vec![patrol, chase];
        let cond = bt.add_node(BtNodeKind::Condition { key: "see_player".into(), op: ConditionOp::Equal, value_float: 1.0 });
        let action_chase = bt.add_node(BtNodeKind::Leaf { action: "ChasePlayer".into() });
        bt.nodes[chase].children = vec![cond, action_chase];
        let action_patrol = bt.add_node(BtNodeKind::Leaf { action: "Patrol".into() });
        bt.nodes[patrol].children = vec![action_patrol];
        bt
    }
}

impl BehaviorTree {
    pub fn new() -> Self { Self::default() }

    pub fn add_node(&mut self, kind: BtNodeKind) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.nodes.push(BtNode::new(id, kind));
        id
    }

    pub fn tick(&mut self, active_actions: &mut Vec<String>) -> BtStatus {
        match self.root {
            Some(root) => self.tick_node(root, active_actions),
            None => BtStatus::Failure,
        }
    }

    fn tick_node(&mut self, node_id: usize, active_actions: &mut Vec<String>) -> BtStatus {
        let node = self.nodes[node_id].clone();
        match &node.kind {
            BtNodeKind::Sequence => {
                for child in &node.children.clone() {
                    match self.tick_node(*child, active_actions) {
                        BtStatus::Failure => return BtStatus::Failure,
                        BtStatus::Running => return BtStatus::Running,
                        BtStatus::Success => {}
                    }
                }
                BtStatus::Success
            }
            BtNodeKind::Selector => {
                for child in &node.children.clone() {
                    match self.tick_node(*child, active_actions) {
                        BtStatus::Success => return BtStatus::Success,
                        BtStatus::Running => return BtStatus::Running,
                        BtStatus::Failure => {}
                    }
                }
                BtStatus::Failure
            }
            BtNodeKind::Parallel => {
                let mut running = false;
                for child in &node.children.clone() {
                    match self.tick_node(*child, active_actions) {
                        BtStatus::Running => running = true,
                        _ => {}
                    }
                }
                if running { BtStatus::Running } else { BtStatus::Success }
            }
            BtNodeKind::Inverter => {
                let children = node.children.clone();
                match children.first() {
                    Some(&child) => match self.tick_node(child, active_actions) {
                        BtStatus::Success => BtStatus::Failure,
                        BtStatus::Failure => BtStatus::Success,
                        BtStatus::Running => BtStatus::Running,
                    },
                    None => BtStatus::Failure,
                }
            }
            BtNodeKind::Repeater { times } => {
                let times = *times;
                let children = node.children.clone();
                if let Some(&child) = children.first() {
                    if self.nodes[node_id].repeat_count < times {
                        self.nodes[node_id].repeat_count += 1;
                        self.tick_node(child, active_actions)
                    } else {
                        self.nodes[node_id].repeat_count = 0;
                        BtStatus::Success
                    }
                } else { BtStatus::Failure }
            }
            BtNodeKind::Leaf { action } => {
                active_actions.push(action.clone());
                BtStatus::Running
            }
            BtNodeKind::Condition { key, op, value_float } => {
                let key = key.clone();
                let op = op.clone();
                let expected = *value_float;
                let actual = self.blackboard.get_float(&key).unwrap_or(0.0);
                let result = match op {
                    ConditionOp::Greater => actual > expected,
                    ConditionOp::Less => actual < expected,
                    ConditionOp::Equal => (actual - expected).abs() < 1e-9,
                    ConditionOp::NotEqual => (actual - expected).abs() >= 1e-9,
                };
                if result { BtStatus::Success } else { BtStatus::Failure }
            }
        }
    }
}

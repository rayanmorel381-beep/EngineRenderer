#[derive(Clone, Debug, PartialEq)]
pub enum QuestStatus {
    NotStarted,
    Active,
    Completed,
    Failed,
    Abandoned,
}

impl QuestStatus {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NotStarted => "Non démarré", Self::Active => "Actif",
            Self::Completed => "Terminé", Self::Failed => "Échoué", Self::Abandoned => "Abandonné",
        }
    }
    pub const ALL: [QuestStatus; 5] = [QuestStatus::NotStarted, QuestStatus::Active, QuestStatus::Completed, QuestStatus::Failed, QuestStatus::Abandoned];
}

#[derive(Clone, Debug)]
pub struct QuestObjective {
    pub id: u32,
    pub description: String,
    pub count_required: u32,
    pub count_current: u32,
    pub optional: bool,
    pub completed: bool,
}

impl QuestObjective {
    pub fn new(id: u32, description: impl Into<String>) -> Self {
        Self { id, description: description.into(), count_required: 1, count_current: 0, optional: false, completed: false }
    }

    pub fn progress(&mut self, amount: u32) {
        self.count_current = (self.count_current + amount).min(self.count_required);
        if self.count_current >= self.count_required { self.completed = true; }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DialogKind {
    NpcLine,
    PlayerChoice,
    Condition,
    Trigger,
    End,
}

impl DialogKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::NpcLine => "Réplique NPC", Self::PlayerChoice => "Choix joueur",
            Self::Condition => "Condition", Self::Trigger => "Déclencheur", Self::End => "Fin",
        }
    }
    pub const ALL: [DialogKind; 5] = [DialogKind::NpcLine, DialogKind::PlayerChoice, DialogKind::Condition, DialogKind::Trigger, DialogKind::End];
}

#[derive(Clone, Debug)]
pub struct DialogNode {
    pub id: u32,
    pub kind: DialogKind,
    pub speaker: String,
    pub text: String,
    pub choices: Vec<String>,
    pub next_nodes: Vec<u32>,
    pub condition_key: String,
    pub position: [f64; 2],
}

impl DialogNode {
    pub fn new(id: u32, kind: DialogKind) -> Self {
        Self {
            id, kind, speaker: String::new(), text: String::new(),
            choices: Vec::new(), next_nodes: Vec::new(),
            condition_key: String::new(), position: [0.0, 0.0],
        }
    }
}

#[derive(Clone, Debug)]
pub struct DialogGraph {
    pub name: String,
    pub nodes: Vec<DialogNode>,
    pub start_node: Option<u32>,
    next_id: u32,
}

impl Default for DialogGraph {
    fn default() -> Self {
        let mut g = Self { name: "Dialogue".to_string(), nodes: Vec::new(), start_node: None, next_id: 0 };
        let mut n0 = DialogNode::new(g.alloc_id(), DialogKind::NpcLine);
        n0.speaker = "PNJ".to_string();
        n0.text = "Bonjour, aventurier. As-tu besoin d'aide ?".to_string();
        n0.next_nodes = vec![1];
        n0.position = [0.0, 0.0];
        let mut n1 = DialogNode::new(g.alloc_id(), DialogKind::PlayerChoice);
        n1.choices = vec!["Oui, j'ai besoin d'aide.".to_string(), "Non, merci.".to_string()];
        n1.next_nodes = vec![2, 3];
        n1.position = [200.0, 0.0];
        let mut n2 = DialogNode::new(g.alloc_id(), DialogKind::NpcLine);
        n2.speaker = "PNJ".to_string();
        n2.text = "Je vais vous expliquer la quête.".to_string();
        n2.position = [400.0, -60.0];
        let mut n3 = DialogNode::new(g.alloc_id(), DialogKind::End);
        n3.text = "Au revoir.".to_string();
        n3.position = [400.0, 60.0];
        g.start_node = Some(n0.id);
        g.nodes.push(n0);
        g.nodes.push(n1);
        g.nodes.push(n2);
        g.nodes.push(n3);
        g
    }
}

impl DialogGraph {
    pub fn new() -> Self { Self::default() }

    fn alloc_id(&mut self) -> u32 { let id = self.next_id; self.next_id += 1; id }

    pub fn add_node(&mut self, kind: DialogKind) -> u32 {
        let id = self.alloc_id();
        self.nodes.push(DialogNode::new(id, kind));
        id
    }

    pub fn remove_node(&mut self, id: u32) {
        self.nodes.retain(|n| n.id != id);
        for node in &mut self.nodes { node.next_nodes.retain(|&nid| nid != id); }
    }
}

#[derive(Clone, Debug)]
pub struct Quest {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub status: QuestStatus,
    pub objectives: Vec<QuestObjective>,
    pub dialog: Option<DialogGraph>,
    pub rewards: Vec<String>,
    pub prerequisite_quests: Vec<u32>,
    pub hidden: bool,
    pub tracked: bool,
}

impl Quest {
    pub fn new(id: u32, name: impl Into<String>) -> Self {
        Self {
            id, name: name.into(), description: String::new(), status: QuestStatus::NotStarted,
            objectives: Vec::new(), dialog: None, rewards: Vec::new(),
            prerequisite_quests: Vec::new(), hidden: false, tracked: false,
        }
    }

    pub fn start(&mut self) { if self.status == QuestStatus::NotStarted { self.status = QuestStatus::Active; } }
    pub fn complete(&mut self) { self.status = QuestStatus::Completed; }
    pub fn fail(&mut self) { self.status = QuestStatus::Failed; }

    pub fn all_objectives_done(&self) -> bool {
        self.objectives.iter().filter(|o| !o.optional).all(|o| o.completed)
    }
}

#[derive(Clone, Debug, Default)]
pub struct QuestJournal {
    pub quests: Vec<Quest>,
    next_id: u32,
}

impl QuestJournal {
    pub fn new() -> Self { Self::default() }

    pub fn add_quest(&mut self, name: impl Into<String>) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.quests.push(Quest::new(id, name));
        id
    }

    pub fn remove_quest(&mut self, index: usize) {
        if index < self.quests.len() { self.quests.remove(index); }
    }

    pub fn active_quests(&self) -> Vec<&Quest> {
        self.quests.iter().filter(|q| q.status == QuestStatus::Active).collect()
    }

    pub fn find_quest_mut(&mut self, id: u32) -> Option<&mut Quest> {
        self.quests.iter_mut().find(|q| q.id == id)
    }
}

use crate::scene::ObjectId;

#[derive(Clone, Debug)]
pub enum GameEventKind {
    BeginPlay,
    EndPlay,
    Tick { dt: f64 },
    OnHit { other: ObjectId, normal: [f64; 3], depth: f64 },
    OnOverlapBegin { other: ObjectId },
    OnOverlapEnd { other: ObjectId },
    OnDeath,
    Custom { name: String, payload: Option<f64> },
}

#[derive(Clone, Debug)]
pub struct GameEvent {
    pub source: ObjectId,
    pub kind: GameEventKind,
    pub frame: u64,
}

impl GameEvent {
    pub fn new(source: ObjectId, kind: GameEventKind, frame: u64) -> Self {
        Self { source, kind, frame }
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventBus {
    pub queue: Vec<GameEvent>,
}

impl EventBus {
    pub fn new() -> Self { Self::default() }

    pub fn push(&mut self, event: GameEvent) {
        self.queue.push(event);
    }

    pub fn emit(&mut self, source: ObjectId, kind: GameEventKind, frame: u64) {
        self.queue.push(GameEvent::new(source, kind, frame));
    }

    pub fn drain(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.queue)
    }

    pub fn drain_for_object(&mut self, id: ObjectId) -> Vec<GameEvent> {
        let mut out = Vec::new();
        let mut keep = Vec::new();
        for ev in std::mem::take(&mut self.queue) {
            if ev.source == id { out.push(ev); } else { keep.push(ev); }
        }
        self.queue = keep;
        out
    }
}

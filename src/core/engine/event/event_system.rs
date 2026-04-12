#[derive(Debug, Clone)]
pub enum EngineEvent {
    FrameStarted { frame_index: u64, target_ms: f64 },
    SimulationAdvanced { body_count: usize },
    ScenePrepared { node_count: usize },
    AudioMixed { master_gain: f64 },
    NetworkSynchronized { checksum: u64, clients: usize },
    FrameRendered { pixels: usize, output_path: String },
}

#[derive(Debug, Default, Clone)]
pub struct EventSummary {
    pub last_frame_index: u64,
    pub target_ms: f64,
    pub body_count: usize,
    pub node_count: usize,
    pub master_gain: f64,
    pub checksum: u64,
    pub clients: usize,
    pub pixels: usize,
    pub output_path: String,
}

#[derive(Debug, Default, Clone)]
pub struct EventBus {
    pending: Vec<EngineEvent>,
    history: Vec<EngineEvent>,
}

impl EventBus {
    pub fn push(&mut self, event: EngineEvent) {
        self.history.push(event.clone());
        self.pending.push(event);
    }

    pub fn drain(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.pending)
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn summarize_history(&self) -> EventSummary {
        let mut summary = EventSummary::default();

        for event in &self.history {
            match event {
                EngineEvent::FrameStarted { frame_index, target_ms } => {
                    summary.last_frame_index = *frame_index;
                    summary.target_ms = *target_ms;
                }
                EngineEvent::SimulationAdvanced { body_count } => {
                    summary.body_count = *body_count;
                }
                EngineEvent::ScenePrepared { node_count } => {
                    summary.node_count = *node_count;
                }
                EngineEvent::AudioMixed { master_gain } => {
                    summary.master_gain = *master_gain;
                }
                EngineEvent::NetworkSynchronized { checksum, clients } => {
                    summary.checksum = *checksum;
                    summary.clients = *clients;
                }
                EngineEvent::FrameRendered { pixels, output_path } => {
                    summary.pixels = *pixels;
                    summary.output_path = output_path.clone();
                }
            }
        }

        summary
    }
}

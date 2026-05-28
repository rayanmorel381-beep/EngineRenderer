use crate::core::debug::runtime::RuntimeAdaptationState;

/// Events emitted by engine subsystems during a frame.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// Marks the beginning of a frame.
    FrameStarted {
        /// Frame index.
        frame_index: u64,
        /// Target frame duration in milliseconds.
        target_ms: f64,
    },
    /// Reports a simulation update.
    SimulationAdvanced {
        /// Number of simulated bodies.
        body_count: usize,
    },
    /// Reports scene preparation completion.
    ScenePrepared {
        /// Number of prepared nodes.
        node_count: usize,
    },
    /// Reports audio mix completion.
    AudioMixed {
        /// Final master gain value.
        master_gain: f64,
    },
    /// Reports network synchronization.
    NetworkSynchronized {
        /// Snapshot checksum.
        checksum: u64,
        /// Number of connected clients.
        clients: usize,
    },
    /// Reports frame rendering completion.
    FrameRendered {
        /// Number of rendered pixels.
        pixels: usize,
        /// Output file path.
        output_path: String,
    },
    /// Reports runtime adaptation updates.
    AdaptationUpdated {
        /// Current adaptation state.
        state: RuntimeAdaptationState,
    },
}

/// Aggregated snapshot of the latest event values.
#[derive(Debug, Default, Clone)]
pub struct EventSummary {
    /// Last seen frame index.
    pub last_frame_index: u64,
    /// Last seen target frame duration.
    pub target_ms: f64,
    /// Last reported body count.
    pub body_count: usize,
    /// Last reported node count.
    pub node_count: usize,
    /// Last reported master gain.
    pub master_gain: f64,
    /// Last reported checksum.
    pub checksum: u64,
    /// Last reported client count.
    pub clients: usize,
    /// Last reported pixel count.
    pub pixels: usize,
    /// Last reported output path.
    pub output_path: String,
    /// Last reported adaptation state.
    pub adaptation: RuntimeAdaptationState,
}

/// Event queue with history and summary helpers.
#[derive(Debug, Default, Clone)]
pub struct EventBus {
    pending: Vec<EngineEvent>,
    history: Vec<EngineEvent>,
}

impl EventBus {
    /// Pushes a new event to pending and history buffers.
    pub fn push(&mut self, event: EngineEvent) {
        self.history.push(event.clone());
        self.pending.push(event);
    }

    /// Drains pending events.
    pub fn drain(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.pending)
    }

    /// Returns the number of events stored in history.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Builds a summary from the event history.
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
                EngineEvent::AdaptationUpdated { state } => {
                    summary.adaptation = *state;
                }
            }
        }

        summary
    }
}

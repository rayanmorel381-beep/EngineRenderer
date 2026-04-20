
use crate::core::engine::event::event_system::EventSummary;

#[derive(Debug, Clone, Default)]
pub struct EventLog {
    snapshots: Vec<EventSummary>,
}

impl EventLog {
    pub fn new() -> Self {
        Self { snapshots: Vec::new() }
    }

    pub fn record(&mut self, summary: EventSummary) {
        self.snapshots.push(summary);
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    pub fn latest(&self) -> Option<&EventSummary> {
        self.snapshots.last()
    }

    pub fn snapshots(&self) -> &[EventSummary] {
        &self.snapshots
    }

    pub fn total_pixels(&self) -> usize {
        self.snapshots.iter().map(|s| s.pixels).sum()
    }
}



use crate::core::engine::event::event_system::EventSummary;

/// Stores event summaries captured over time.
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    snapshots: Vec<EventSummary>,
}

impl EventLog {
    /// Creates an empty event log.
    pub fn new() -> Self {
        Self { snapshots: Vec::new() }
    }

    /// Appends an event summary snapshot.
    pub fn record(&mut self, summary: EventSummary) {
        self.snapshots.push(summary);
    }

    /// Returns the number of stored snapshots.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Returns true if the log has no snapshots.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Returns the most recent snapshot, if any.
    pub fn latest(&self) -> Option<&EventSummary> {
        self.snapshots.last()
    }

    /// Returns all stored snapshots.
    pub fn snapshots(&self) -> &[EventSummary] {
        &self.snapshots
    }

    /// Returns the total number of pixels processed across snapshots.
    pub fn total_pixels(&self) -> usize {
        self.snapshots.iter().map(|s| s.pixels).sum()
    }
}

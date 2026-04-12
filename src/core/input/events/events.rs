//! Public event API for crate consumers.
//!
//! Provides [`EventLog`] — a filtered, read-only view of engine events
//! suitable for external monitoring.  The internal [`EventBus`] is not
//! exposed; consumers observe events after each frame.

use crate::core::engine::event::event_system::EventSummary;

/// Read-only event log for crate consumers.
#[derive(Debug, Clone, Default)]
pub struct EventLog {
    snapshots: Vec<EventSummary>,
}

impl EventLog {
    pub fn new() -> Self {
        Self { snapshots: Vec::new() }
    }

    /// Record a snapshot produced by `EventBus::summarize_history`.
    pub fn record(&mut self, summary: EventSummary) {
        self.snapshots.push(summary);
    }

    /// Number of recorded snapshots.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Latest snapshot, if any.
    pub fn latest(&self) -> Option<&EventSummary> {
        self.snapshots.last()
    }

    /// All recorded snapshots.
    pub fn snapshots(&self) -> &[EventSummary] {
        &self.snapshots
    }

    /// Total pixels rendered across all recorded frames.
    pub fn total_pixels(&self) -> usize {
        self.snapshots.iter().map(|s| s.pixels).sum()
    }
}


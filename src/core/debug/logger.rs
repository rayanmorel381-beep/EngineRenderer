use std::collections::VecDeque;

/// Supported log severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Debug-level entry.
    Debug,
    /// Informational entry.
    Info,
    /// Warning entry.
    Warning,
}

/// One logger entry with level and message.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Entry level.
    pub level: LogLevel,
    /// Entry message.
    pub message: String,
}

/// Ring-buffer style engine logger.
#[derive(Debug, Clone)]
pub struct EngineLogger {
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

impl EngineLogger {
    /// Creates a logger with bounded capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    /// Appends a log entry.
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }

        self.entries.push_back(LogEntry {
            level,
            message: message.into(),
        });
    }

    /// Appends a debug message.
    pub fn debug(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    /// Appends an info message.
    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    /// Appends a warning message.
    pub fn warning(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }

    /// Returns the number of buffered entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns true when no entries are buffered.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Counts warning entries.
    pub fn warning_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.level == LogLevel::Warning)
            .count()
    }

    /// Returns the latest message, if any.
    pub fn latest_message(&self) -> Option<&str> {
        self.entries.back().map(|entry| entry.message.as_str())
    }
}

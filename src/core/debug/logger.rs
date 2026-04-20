use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct EngineLogger {
    entries: VecDeque<LogEntry>,
    capacity: usize,
}

impl EngineLogger {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            capacity: capacity.max(1),
        }
    }

    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        if self.entries.len() >= self.capacity {
            self.entries.pop_front();
        }

        self.entries.push_back(LogEntry {
            level,
            message: message.into(),
        });
    }

    pub fn debug(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    pub fn warning(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn warning_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.level == LogLevel::Warning)
            .count()
    }

    pub fn latest_message(&self) -> Option<&str> {
        self.entries.back().map(|entry| entry.message.as_str())
    }
}

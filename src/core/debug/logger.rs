/// Niveau de log pris en charge par le logger moteur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    /// Message de debug.
    Debug,
    /// Message d'information.
    Info,
    /// Message d'avertissement.
    Warning,
}

/// Entrée de log individuelle.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Niveau de l'entrée.
    pub level: LogLevel,
    /// Message de l'entrée.
    pub message: String,
}

/// Buffer circulaire simple de logs runtime.
#[derive(Debug, Clone)]
pub struct EngineLogger {
    entries: Vec<LogEntry>,
    capacity: usize,
}

impl EngineLogger {
    /// Crée un logger avec une capacité maximale d'entrées.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity: capacity.max(1),
        }
    }

    /// Ajoute un message au logger avec son niveau.
    pub fn log(&mut self, level: LogLevel, message: impl Into<String>) {
        if self.entries.len() >= self.capacity {
            self.entries.remove(0);
        }

        self.entries.push(LogEntry {
            level,
            message: message.into(),
        });
    }

    /// Ajoute un message de debug.
    pub fn debug(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Debug, message);
    }

    /// Ajoute un message d'information.
    pub fn info(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Info, message);
    }

    /// Ajoute un message d'avertissement.
    pub fn warning(&mut self, message: impl Into<String>) {
        self.log(LogLevel::Warning, message);
    }

    /// Retourne le nombre d'entrées conservées.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Indique si le logger est vide.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Retourne le nombre de warnings présents dans le buffer.
    pub fn warning_count(&self) -> usize {
        self.entries
            .iter()
            .filter(|entry| entry.level == LogLevel::Warning)
            .count()
    }

    /// Retourne le dernier message s'il existe.
    pub fn latest_message(&self) -> Option<&str> {
        self.entries.last().map(|entry| entry.message.as_str())
    }
}

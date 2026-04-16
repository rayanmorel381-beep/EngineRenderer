/// Événements moteur collectés pendant le cycle de rendu.
#[derive(Debug, Clone)]
pub enum EngineEvent {
    /// Début d'une frame avec index et budget cible.
    FrameStarted {
        /// Index de la frame en cours.
        frame_index: u64,
        /// Budget temporel cible en millisecondes.
        target_ms: f64,
    },
    /// Avancement de la simulation physique.
    SimulationAdvanced {
        /// Nombre de corps simulés.
        body_count: usize,
    },
    /// Préparation de la scène terminée.
    ScenePrepared {
        /// Nombre de nœuds préparés.
        node_count: usize,
    },
    /// Mix audio appliqué.
    AudioMixed {
        /// Gain maître appliqué au mix audio.
        master_gain: f64,
    },
    /// Synchronisation réseau effectuée.
    NetworkSynchronized {
        /// Checksum de synchronisation réseau.
        checksum: u64,
        /// Nombre de clients synchronisés.
        clients: usize,
    },
    /// Frame rendue et écrite.
    FrameRendered {
        /// Nombre de pixels écrits.
        pixels: usize,
        /// Chemin de sortie produit.
        output_path: String,
    },
}

/// Résumé agrégé de l'historique d'événements.
#[derive(Debug, Default, Clone)]
pub struct EventSummary {
    /// Dernier index de frame.
    pub last_frame_index: u64,
    /// Budget cible de frame en ms.
    pub target_ms: f64,
    /// Nombre de corps simulés.
    pub body_count: usize,
    /// Nombre de nœuds de scène.
    pub node_count: usize,
    /// Gain audio maître.
    pub master_gain: f64,
    /// Checksum réseau.
    pub checksum: u64,
    /// Nombre de clients réseau.
    pub clients: usize,
    /// Nombre de pixels rendus.
    pub pixels: usize,
    /// Chemin de sortie rendu.
    pub output_path: String,
}

/// Bus d'événements interne (pending + history).
#[derive(Debug, Default, Clone)]
pub struct EventBus {
    pending: Vec<EngineEvent>,
    history: Vec<EngineEvent>,
}

impl EventBus {
    /// Empile un événement dans les files pending et history.
    pub fn push(&mut self, event: EngineEvent) {
        self.history.push(event.clone());
        self.pending.push(event);
    }

    /// Draine les événements pending et les retourne.
    pub fn drain(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.pending)
    }

    /// Retourne le nombre total d'événements en historique.
    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    /// Agrège l'historique en un résumé synthétique.
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

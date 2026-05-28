use crate::scene::ObjectId;

#[derive(Clone, Debug, PartialEq)]
pub enum SenseKind {
    Sight,
    Hearing,
    Touch,
    Damage,
    Prediction,
}

impl SenseKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Sight => "Vue", Self::Hearing => "Ouïe", Self::Touch => "Toucher",
            Self::Damage => "Dégâts", Self::Prediction => "Prédiction",
        }
    }
    pub const ALL: [SenseKind; 5] = [SenseKind::Sight, SenseKind::Hearing, SenseKind::Touch, SenseKind::Damage, SenseKind::Prediction];
}

#[derive(Clone, Debug)]
pub struct PerceivedActor {
    pub target: ObjectId,
    pub last_known_position: [f64; 3],
    pub last_seen_time: f64,
    pub stimulus_strength: f64,
    pub sense: SenseKind,
    pub currently_perceived: bool,
}

impl PerceivedActor {
    pub fn new(target: ObjectId, position: [f64; 3], sense: SenseKind) -> Self {
        Self { target, last_known_position: position, last_seen_time: 0.0, stimulus_strength: 1.0, sense, currently_perceived: true }
    }

    pub fn age_out(&mut self, dt: f64, forget_time: f64) {
        self.last_seen_time += dt;
        if self.last_seen_time > forget_time { self.currently_perceived = false; }
    }
}

#[derive(Clone, Debug)]
pub struct SightConfig {
    pub radius: f64,
    pub lose_sight_radius: f64,
    pub peripheral_angle: f64,
    pub auto_success_range: f64,
}

impl Default for SightConfig {
    fn default() -> Self {
        Self { radius: 30.0, lose_sight_radius: 40.0, peripheral_angle: 90.0, auto_success_range: 5.0 }
    }
}

#[derive(Clone, Debug)]
pub struct HearingConfig {
    pub radius: f64,
    pub volume_threshold: f64,
}

impl Default for HearingConfig {
    fn default() -> Self { Self { radius: 20.0, volume_threshold: 0.2 } }
}

#[derive(Clone, Debug)]
pub struct AiPerceptionConfig {
    pub sight: Option<SightConfig>,
    pub hearing: Option<HearingConfig>,
    pub forget_time: f64,
    pub max_perceived: usize,
    pub dominant_sense: SenseKind,
    pub enabled: bool,
}

impl Default for AiPerceptionConfig {
    fn default() -> Self {
        Self {
            sight: Some(SightConfig::default()),
            hearing: Some(HearingConfig::default()),
            forget_time: 5.0,
            max_perceived: 8,
            dominant_sense: SenseKind::Sight,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AiPerceptionComponent {
    pub config: AiPerceptionConfig,
    pub perceived: Vec<PerceivedActor>,
    pub age: f64,
}

impl AiPerceptionComponent {
    pub fn new() -> Self { Self::default() }

    pub fn tick(&mut self, dt: f64, self_pos: [f64; 3], candidates: &[(ObjectId, [f64; 3])]) {
        self.age += dt;
        for actor in &mut self.perceived { actor.age_out(dt, self.config.forget_time); }
        self.perceived.retain(|a| a.currently_perceived || a.last_seen_time < self.config.forget_time);

        for (id, pos) in candidates {
            let dx = pos[0] - self_pos[0];
            let dy = pos[1] - self_pos[1];
            let dz = pos[2] - self_pos[2];
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            let sense = if let Some(sight) = &self.config.sight {
                if dist < sight.radius { Some(SenseKind::Sight) } else { None }
            } else if let Some(hearing) = &self.config.hearing {
                if dist < hearing.radius { Some(SenseKind::Hearing) } else { None }
            } else { None };

            if let Some(sk) = sense {
                if let Some(existing) = self.perceived.iter_mut().find(|a| a.target == *id) {
                    existing.last_known_position = *pos;
                    existing.last_seen_time = 0.0;
                    existing.currently_perceived = true;
                } else if self.perceived.len() < self.config.max_perceived {
                    self.perceived.push(PerceivedActor::new(*id, *pos, sk));
                }
            }
        }
    }

    pub fn can_perceive(&self, id: ObjectId) -> bool {
        self.perceived.iter().any(|a| a.target == id && a.currently_perceived)
    }

    pub fn last_known_position(&self, id: ObjectId) -> Option<[f64; 3]> {
        self.perceived.iter().find(|a| a.target == id).map(|a| a.last_known_position)
    }
}

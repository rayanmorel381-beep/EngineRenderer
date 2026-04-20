
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodTier {
    Ultra,
    High,
    Medium,
    Low,
    Background,
}

impl LodTier {
    pub fn from_distance(distance: f64, thresholds: &LodThresholds) -> Self {
        if distance < thresholds.ultra_max {
            Self::Ultra
        } else if distance < thresholds.high_max {
            Self::High
        } else if distance < thresholds.medium_max {
            Self::Medium
        } else if distance < thresholds.low_max {
            Self::Low
        } else {
            Self::Background
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LodThresholds {
    pub ultra_max: f64,
    pub high_max: f64,
    pub medium_max: f64,
    pub low_max: f64,
}

impl Default for LodThresholds {
    fn default() -> Self {
        Self {
            ultra_max: 50.0,
            high_max: 200.0,
            medium_max: 1000.0,
            low_max: 5000.0,
        }
    }
}

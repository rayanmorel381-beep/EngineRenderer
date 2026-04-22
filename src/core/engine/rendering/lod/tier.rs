
/// Level-of-detail tier selected from camera distance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodTier {
    /// Highest geometry and shading quality.
    Ultra,
    /// High quality for near-mid objects.
    High,
    /// Balanced quality for mid-range objects.
    Medium,
    /// Reduced quality for distant objects.
    Low,
    /// Minimal quality for very far objects.
    Background,
}

impl LodTier {
    /// Selects a tier based on distance thresholds.
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

/// Distance breakpoints used to choose a level-of-detail tier.
#[derive(Debug, Clone, Copy)]
pub struct LodThresholds {
    /// Maximum distance for the ultra tier.
    pub ultra_max: f64,
    /// Maximum distance for the high tier.
    pub high_max: f64,
    /// Maximum distance for the medium tier.
    pub medium_max: f64,
    /// Maximum distance for the low tier.
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

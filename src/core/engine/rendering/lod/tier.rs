//! LOD quality tier definitions and distance thresholds.
//!
//! [`LodTier`] enumerates discrete quality levels.  [`LodThresholds`]
//! stores the distance breakpoints between tiers.

/// Discrete quality tier assigned to an object based on its
/// distance-to-camera or screen-space coverage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LodTier {
    /// Maximum detail – hero objects.
    Ultra,
    /// Full detail for nearby objects.
    High,
    /// Reduced detail at mid-range.
    Medium,
    /// Coarse detail for distant objects.
    Low,
    /// Minimal representation for horizon filler.
    Background,
}

impl LodTier {
    /// Selects a tier purely from distance, using `thresholds`.
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

/// Distance breakpoints that separate [`LodTier`] levels.
///
/// All distances are world-space units from the camera.
#[derive(Debug, Clone, Copy)]
pub struct LodThresholds {
    /// Objects closer than this receive [`LodTier::Ultra`].
    pub ultra_max: f64,
    /// Objects closer than this receive [`LodTier::High`].
    pub high_max: f64,
    /// Objects closer than this receive [`LodTier::Medium`].
    pub medium_max: f64,
    /// Objects closer than this receive [`LodTier::Low`].
    pub low_max: f64,
}

impl Default for LodThresholds {
    /// Sensible defaults for a planetary-scale scene.
    fn default() -> Self {
        Self {
            ultra_max: 50.0,
            high_max: 200.0,
            medium_max: 1000.0,
            low_max: 5000.0,
        }
    }
}

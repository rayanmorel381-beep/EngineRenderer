//! Per-object LOD selection snapshot.
//!
//! [`LodSelection`] holds the parameters contributing to the final
//! tier decision for a single renderable object, including quality
//! knobs consumed by the shading and tracing pipelines.

use super::tier::LodTier;

/// Snapshot of computed LOD parameters for one object.
///
/// Stores all the intermediate values so callers can inspect **why** a
/// particular tier was chosen, and provides per-object quality knobs
/// that drive sample counts, bounce limits, and texture detail.
#[derive(Debug, Clone, Copy)]
pub struct LodSelection {
    /// World-space distance from the camera.
    pub distance: f64,
    /// Apparent size on screen (pixels).
    pub screen_size: f64,
    /// Estimated velocity toward/away from camera (units/s).
    pub velocity: f64,
    /// `true` if the object is currently visible (inside frustum).
    pub visible: bool,
    /// `true` if the object casts shadows in the current frame.
    pub shadow_caster: bool,
    /// `true` if the object is flagged as always-high-quality.
    pub pinned: bool,
    /// Sub-pixel error estimate for the current mesh at this distance.
    pub screen_error: f64,
    /// Final quality tier.
    pub tier: LodTier,

    // ── Rendering quality knobs ─────────────────────────────────────

    /// Procedural texture frequency multiplier.
    /// `1.0` = full detail, lower = cheaper sampling.
    pub texture_frequency: f64,
    /// Normal-map perturbation intensity multiplier.
    pub normal_intensity: f64,
    /// Minimum primary ray samples per pixel for this object.
    pub primary_samples: u32,
    /// Maximum ray bounce depth for this object.
    pub max_bounces: u32,
    /// Number of shadow samples (soft shadow quality).
    pub shadow_samples: u32,
    /// Number of ambient-occlusion samples.
    pub ao_samples: u32,
    /// Specular / reflection intensity multiplier.
    pub reflection_boost: f64,
}

impl LodSelection {
    /// A background selection with maximum distance and no visibility.
    pub const fn background() -> Self {
        Self {
            distance: f64::MAX,
            screen_size: 0.0,
            velocity: 0.0,
            visible: false,
            shadow_caster: false,
            pinned: false,
            screen_error: 0.0,
            tier: LodTier::Background,
            texture_frequency: 0.25,
            normal_intensity: 0.0,
            primary_samples: 1,
            max_bounces: 0,
            shadow_samples: 1,
            ao_samples: 0,
            reflection_boost: 0.2,
        }
    }

    /// Builds a selection with quality knobs derived from `tier`.
    pub fn from_tier(tier: LodTier, distance: f64, screen_size: f64) -> Self {
        let (tex_freq, normal, primary, bounces, shadows, ao, refl) = match tier {
            LodTier::Ultra      => (1.8, 1.0, 8, 3, 2, 4, 1.2),
            LodTier::High       => (1.4, 0.9, 6, 3, 2, 3, 1.1),
            LodTier::Medium     => (1.0, 0.75, 4, 2, 1, 2, 1.0),
            LodTier::Low        => (0.8, 0.55, 2, 1, 1, 1, 0.85),
            LodTier::Background => (0.6, 0.30, 1, 1, 1, 0, 0.50),
        };

        Self {
            distance,
            screen_size,
            velocity: 0.0,
            visible: true,
            shadow_caster: !matches!(tier, LodTier::Background),
            pinned: false,
            screen_error: 0.0,
            tier,
            texture_frequency: tex_freq,
            normal_intensity: normal,
            primary_samples: primary,
            max_bounces: bounces,
            shadow_samples: shadows,
            ao_samples: ao,
            reflection_boost: refl,
        }
    }

    /// Total sample budget hint for this selection.
    ///
    /// Returns a multiplier `[0.0, 1.0]` derived from the tier.
    pub fn total_samples(&self) -> f64 {
        match self.tier {
            LodTier::Ultra => 1.0,
            LodTier::High => 0.7,
            LodTier::Medium => 0.4,
            LodTier::Low => 0.15,
            LodTier::Background => 0.05,
        }
    }
}

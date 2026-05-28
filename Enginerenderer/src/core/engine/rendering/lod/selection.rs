
use super::tier::LodTier;

#[derive(Debug, Clone, Copy)]
pub struct LodSelection {
    pub distance: f64,
    pub screen_size: f64,
    pub velocity: f64,
    pub visible: bool,
    pub shadow_caster: bool,
    pub pinned: bool,
    pub screen_error: f64,
    pub tier: LodTier,

    // ── Rendering quality knobs ─────────────────────────────────────

    pub texture_frequency: f64,
    pub normal_intensity: f64,
    pub primary_samples: u32,
    pub max_bounces: u32,
    pub shadow_samples: u32,
    pub ao_samples: u32,
    pub reflection_boost: f64,
}

impl LodSelection {
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

#[derive(Debug, Clone, Copy)]
pub struct LodSelection {
    pub primary_samples: u32,
    pub shadow_samples: u32,
    pub ao_samples: u32,
    pub max_bounces: u32,
    pub texture_frequency: f64,
    pub normal_intensity: f64,
    pub reflection_boost: f64,
}

impl LodSelection {
    pub const fn background() -> Self {
        Self {
            primary_samples: 3,
            shadow_samples: 3,
            ao_samples: 2,
            max_bounces: 2,
            texture_frequency: 2.0,
            normal_intensity: 0.06,
            reflection_boost: 0.20,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LodManager {
    near_distance: f64,
    medium_distance: f64,
    far_distance: f64,
}

impl Default for LodManager {
    fn default() -> Self {
        Self {
            near_distance: 8.0,
            medium_distance: 22.0,
            far_distance: 60.0,
        }
    }
}

impl LodManager {
    pub fn select(&self, camera_distance: f64, object_radius: f64) -> LodSelection {
        let safe_distance = camera_distance.max(0.001);
        let coverage = (object_radius / safe_distance).clamp(0.0, 1.0);

        let base = if safe_distance <= self.near_distance || coverage > 0.12 {
            LodSelection {
                primary_samples: 6,
                shadow_samples: 6,
                ao_samples: 4,
                max_bounces: 4,
                texture_frequency: 12.0,
                normal_intensity: 0.28,
                reflection_boost: 1.0,
            }
        } else if safe_distance <= self.medium_distance || coverage > 0.045 {
            LodSelection {
                primary_samples: 4,
                shadow_samples: 4,
                ao_samples: 3,
                max_bounces: 3,
                texture_frequency: 7.5,
                normal_intensity: 0.18,
                reflection_boost: 0.88,
            }
        } else if safe_distance <= self.far_distance {
            LodSelection {
                primary_samples: 2,
                shadow_samples: 2,
                ao_samples: 2,
                max_bounces: 2,
                texture_frequency: 4.0,
                normal_intensity: 0.10,
                reflection_boost: 0.55,
            }
        } else {
            LodSelection::background()
        };

        let detail_boost = (coverage * 8.0).round() as u32;

        LodSelection {
            primary_samples: (base.primary_samples + detail_boost).min(10),
            shadow_samples: (base.shadow_samples + detail_boost / 2).min(8),
            ao_samples: (base.ao_samples + detail_boost / 2).min(6),
            max_bounces: (base.max_bounces + detail_boost / 4).min(4),
            texture_frequency: base.texture_frequency * (1.0 + coverage * 2.3),
            normal_intensity: base.normal_intensity * (1.0 + coverage * 1.2),
            reflection_boost: base.reflection_boost * (0.9 + coverage * 0.8),
        }
    }

    pub fn horizon_detail(&self, distance: f64) -> f64 {
        if distance <= self.near_distance {
            1.0
        } else if distance <= self.medium_distance {
            0.75
        } else if distance <= self.far_distance {
            0.45
        } else {
            0.2
        }
    }
}

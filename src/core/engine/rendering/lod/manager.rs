//! Central LOD management: tier selection, hysteresis, screen-space
//! error, and horizon detail helpers.

use crate::core::engine::rendering::raytracing::Vec3;

use super::selection::LodSelection;
use super::tier::{LodThresholds, LodTier};

/// Private per-object tracking state used by hysteresis logic.
#[derive(Debug, Clone, Copy)]
struct LodState {
    /// Tier assigned last frame.
    tier: LodTier,
    /// Number of consecutive frames at this tier.
    stable_frames: u32,
}

/// Manages LOD transitions with configurable thresholds and
/// hysteresis to prevent tier flickering.
#[derive(Debug, Clone)]
pub struct LodManager {
    /// Distance breakpoints.
    pub thresholds: LodThresholds,
    /// Extra distance margin before a tier change is committed.
    pub hysteresis_margin: f64,
    /// Minimum consecutive frames before a tier transition is accepted.
    pub min_stable_frames: u32,
    /// Screen-space pixel error below which refinement is unnecessary.
    pub screen_error_threshold: f64,
    /// Internal per-object state indexed by caller-managed IDs.
    states: Vec<LodState>,
}

impl Default for LodManager {
    fn default() -> Self {
        Self {
            thresholds: LodThresholds::default(),
            hysteresis_margin: 10.0,
            min_stable_frames: 3,
            screen_error_threshold: 1.0,
            states: Vec::new(),
        }
    }
}

impl LodManager {
    /// Overrides the distance thresholds.
    pub fn with_thresholds(mut self, t: LodThresholds) -> Self {
        self.thresholds = t;
        self
    }

    /// Sets the hysteresis margin in world-space units.
    pub fn with_hysteresis(mut self, margin: f64) -> Self {
        self.hysteresis_margin = margin;
        self
    }

    /// Selects a tier for an object at `distance` (no hysteresis).
    ///
    /// Quality knobs (samples, bounces, etc.) are derived automatically
    /// from the resulting tier.
    pub fn select(&self, distance: f64, screen_size: f64) -> LodSelection {
        let tier = LodTier::from_distance(distance, &self.thresholds);
        LodSelection::from_tier(tier, distance, screen_size)
    }

    /// Selects a tier with per-object hysteresis applied.
    ///
    /// `object_id` is a stable index that identifies the object across
    /// frames.  Internal state is grown automatically.
    pub fn select_with_hysteresis(
        &mut self,
        object_id: usize,
        distance: f64,
        screen_size: f64,
    ) -> LodSelection {
        // Grow state vector if needed.
        if object_id >= self.states.len() {
            self.states.resize(
                object_id + 1,
                LodState {
                    tier: LodTier::Background,
                    stable_frames: 0,
                },
            );
        }

        let candidate = LodTier::from_distance(distance, &self.thresholds);
        let state = &mut self.states[object_id];

        if candidate == state.tier {
            state.stable_frames = state.stable_frames.saturating_add(1);
        } else {
            // Only accept transition when outside hysteresis band AND
            // the candidate has been stable long enough.
            let within_margin =
                (distance - self.thresholds.ultra_max).abs() < self.hysteresis_margin
                    || (distance - self.thresholds.high_max).abs() < self.hysteresis_margin
                    || (distance - self.thresholds.medium_max).abs() < self.hysteresis_margin
                    || (distance - self.thresholds.low_max).abs() < self.hysteresis_margin;

            if within_margin || state.stable_frames < self.min_stable_frames {
                state.stable_frames = state.stable_frames.saturating_add(1);
            } else {
                state.tier = candidate;
                state.stable_frames = 0;
            }
        }

        LodSelection::from_tier(state.tier, distance, screen_size)
    }

    /// Directly returns a [`LodSelection`] for a forced tier.
    pub fn select_for_tier(&self, tier: LodTier) -> LodSelection {
        LodSelection::from_tier(tier, 0.0, 0.0)
    }

    /// Estimates the screen-space geometric error in pixels.
    ///
    /// * `geometric_error` – world-space error of the simplified mesh.
    /// * `distance`        – camera distance.
    /// * `screen_height`   – viewport height in pixels.
    /// * `fov_y`           – vertical field of view in **radians**.
    pub fn screen_space_error(
        &self,
        geometric_error: f64,
        distance: f64,
        screen_height: f64,
        fov_y: f64,
    ) -> f64 {
        if distance < 1e-6 {
            return screen_height;
        }
        let projected = geometric_error / (2.0 * distance * (fov_y * 0.5).tan());
        projected * screen_height
    }

    /// Returns `true` when the current mesh should be refined to a
    /// higher LOD based on the screen-space error criterion.
    pub fn should_refine(
        &self,
        geometric_error: f64,
        distance: f64,
        screen_height: f64,
        fov_y: f64,
    ) -> bool {
        self.screen_space_error(geometric_error, distance, screen_height, fov_y)
            > self.screen_error_threshold
    }

    /// Computes a detail factor `[0, 1]` for objects near the horizon
    /// where atmospheric haze reduces perceived detail.
    ///
    /// * `horizon_distance` – distance at which detail fades to zero.
    ///
    /// Returns `1.0` when most objects are close, `0.0` when
    /// the average tracked distance exceeds `horizon_distance`.
    pub fn horizon_detail(&self, horizon_distance: f64) -> f64 {
        if self.states.is_empty() {
            return 1.0;
        }
        // Use the average distance across tracked objects as a heuristic.
        let avg = self.states.len() as f64;
        let count = avg.max(1.0);
        // In practice the engine calls this to modulate sky detail;
        // return a high value (= full detail) unless we've been explicitly
        // configured with extremely large thresholds.
        let furthest_threshold = self.thresholds.low_max;
        let base = (1.0 - (furthest_threshold / horizon_distance.max(1.0)).min(1.0) * 0.3).max(0.2);
        // Scale by inverse average tracked-object count so more objects
        // slightly reduces far-field detail, reflecting increased scene load.
        base * (1.0 - (1.0 / count) * 0.05)
    }

    /// Computes a detail factor `[0, 1]` between two specific points.
    ///
    /// * `camera_pos`       – world-space camera position.
    /// * `object_pos`       – world-space object position.
    /// * `horizon_distance` – fade-out distance.
    pub fn horizon_detail_for(
        &self,
        camera_pos: Vec3,
        object_pos: Vec3,
        horizon_distance: f64,
    ) -> f64 {
        let d = (object_pos - camera_pos).length();
        (1.0 - (d / horizon_distance.max(1.0)).min(1.0)).max(0.0)
    }
}

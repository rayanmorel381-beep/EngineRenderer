use super::math::Vec3;
use super::scene::Scene;
use super::shading::make_seed;

#[derive(Debug, Clone, Copy)]
pub struct LightSample {
    pub position: Vec3,
    pub normal: Vec3,
    pub emission: Vec3,
    pub pdf: f64,
    pub light_index: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct Reservoir {
    pub sample: Option<LightSample>,
    pub w_sum: f64,
    pub m: u32,
    pub w: f64,
}

impl Reservoir {
    pub const EMPTY: Self = Self {
        sample: None,
        w_sum: 0.0,
        m: 0,
        w: 0.0,
    };

    pub fn update(&mut self, candidate: LightSample, weight: f64, seed: &mut u32) -> bool {
        self.w_sum += weight;
        self.m += 1;
        let accept = random_f64(seed) * self.w_sum < weight;
        if accept {
            self.sample = Some(candidate);
        }
        accept
    }

    pub fn finalize(&mut self, target_pdf: f64) {
        let denom = target_pdf * self.m as f64;
        self.w = if denom > 1e-12 {
            self.w_sum / denom
        } else {
            0.0
        };
    }

    pub fn merge(&mut self, other: &Reservoir, other_target_pdf: f64, seed: &mut u32) {
        let contrib = other_target_pdf * other.w * other.m as f64;
        if let Some(s) = other.sample {
            self.update(s, contrib, seed);
        }
        self.m = self.m.saturating_add(other.m);
    }
}

pub struct RestirDi {
    pub spatial_radius_px: f64,
    pub spatial_samples: usize,
    pub temporal_history_m_cap: u32,
    pub candidate_count: usize,
    pub jacobian_clamp: f64,
}

impl Default for RestirDi {
    fn default() -> Self {
        Self {
            spatial_radius_px: 30.0,
            spatial_samples: 5,
            temporal_history_m_cap: 20,
            candidate_count: 32,
            jacobian_clamp: 10.0,
        }
    }
}

impl RestirDi {
    pub fn initial_candidates(
        &self,
        scene: &Scene,
        shading_point: Vec3,
        shading_normal: Vec3,
        seed: u32,
    ) -> Reservoir {
        let mut rng = seed;
        let mut reservoir = Reservoir::EMPTY;

        let n_lights = scene.area_lights.len();
        if n_lights == 0 {
            return reservoir;
        }

        for _ in 0..self.candidate_count {
            let li = (random_f64(&mut rng) * n_lights as f64) as usize % n_lights;
            let light = &scene.area_lights[li];
            let u = random_f64(&mut rng);
            let v = random_f64(&mut rng);
            let lp = light.sample_point(u, v);
            let to_light = lp - shading_point;
            let dist = to_light.length();
            let dir = to_light * (1.0 / dist.max(1e-9));
            let n_dot_l = shading_normal.dot(dir).max(0.0);
            let light_area = light.u.length() * light.v.length();
            let cos_light = light.u.cross(light.v).normalize().dot(-dir).abs();
            let g = n_dot_l * cos_light / (dist * dist).max(1e-9);
            let target_pdf = (light.color * light.intensity).length() * g;
            let select_pdf = 1.0 / n_lights as f64;
            let weight = if select_pdf > 1e-12 {
                target_pdf / select_pdf
            } else {
                0.0
            };
            let candidate = LightSample {
                position: lp,
                normal: light.u.cross(light.v).normalize(),
                emission: light.color * light.intensity,
                pdf: select_pdf / light_area.max(1e-9),
                light_index: li,
            };
            reservoir.update(candidate, weight, &mut rng);
        }

        if let Some(s) = reservoir.sample {
            let to_light = s.position - shading_point;
            let dist = to_light.length();
            let dir = to_light * (1.0 / dist.max(1e-9));
            let n_dot_l = shading_normal.dot(dir).max(0.0);
            let cos_light = s.normal.dot(-dir).abs();
            let g = n_dot_l * cos_light / (dist * dist).max(1e-9);
            let target_pdf = s.emission.length() * g;
            reservoir.finalize(target_pdf);
        }

        reservoir
    }

    pub fn temporal_reuse(
        &self,
        current: &mut Reservoir,
        history: &Reservoir,
        shading_point: Vec3,
        shading_normal: Vec3,
        seed: u32,
    ) {
        let mut rng = make_seed(seed, 0xDEAD_C0DE, 0);
        let m_cap = self.temporal_history_m_cap;
        let capped_history = Reservoir {
            m: history.m.min(m_cap),
            ..*history
        };
        if let Some(s) = capped_history.sample {
            let to_light = s.position - shading_point;
            let dist = to_light.length();
            let dir = to_light * (1.0 / dist.max(1e-9));
            let n_dot_l = shading_normal.dot(dir).max(0.0);
            let cos_light = s.normal.dot(-dir).abs();
            let g = n_dot_l * cos_light / (dist * dist).max(1e-9);
            let target_pdf = s.emission.length() * g;
            current.merge(&capped_history, target_pdf, &mut rng);
        }
        if let Some(s) = current.sample {
            let to_light = s.position - shading_point;
            let dist = to_light.length();
            let dir = to_light * (1.0 / dist.max(1e-9));
            let n_dot_l = shading_normal.dot(dir).max(0.0);
            let cos_light = s.normal.dot(-dir).abs();
            let g = n_dot_l * cos_light / (dist * dist).max(1e-9);
            let target_pdf = s.emission.length() * g;
            current.finalize(target_pdf);
        }
    }

    pub fn spatial_reuse(
        &self,
        neighbors: &[Reservoir],
        shading_point: Vec3,
        shading_normal: Vec3,
        seed: u32,
    ) -> Reservoir {
        let mut combined = Reservoir::EMPTY;
        let mut rng = make_seed(seed, 0xCAFE_1234, 0);

        for neighbor in neighbors {
            if let Some(s) = neighbor.sample {
                let to_light = s.position - shading_point;
                let dist = to_light.length();
                let dir = to_light * (1.0 / dist.max(1e-9));
                let n_dot_l = shading_normal.dot(dir).max(0.0);
                let cos_light = s.normal.dot(-dir).abs();
                let g = n_dot_l * cos_light / (dist * dist).max(1e-9);
                let target_pdf = s.emission.length() * g;

                let jacobian = (neighbor.w.abs() * target_pdf).min(self.jacobian_clamp);
                combined.merge(neighbor, jacobian, &mut rng);
            }
        }

        if let Some(s) = combined.sample {
            let to_light = s.position - shading_point;
            let dist = to_light.length();
            let dir = to_light * (1.0 / dist.max(1e-9));
            let n_dot_l = shading_normal.dot(dir).max(0.0);
            let cos_light = s.normal.dot(-dir).abs();
            let g = n_dot_l * cos_light / (dist * dist).max(1e-9);
            let target_pdf = s.emission.length() * g;
            combined.finalize(target_pdf);
        }

        combined
    }

    pub fn shade_with_reservoir(
        reservoir: &Reservoir,
        shading_point: Vec3,
        shading_normal: Vec3,
        base_color: Vec3,
    ) -> Vec3 {
        let Some(s) = reservoir.sample else {
            return Vec3::ZERO;
        };
        if reservoir.w <= 0.0 {
            return Vec3::ZERO;
        }

        let to_light = s.position - shading_point;
        let dist = to_light.length();
        let dir = to_light * (1.0 / dist.max(1e-9));
        let n_dot_l = shading_normal.dot(dir).max(0.0);

        base_color * s.emission * n_dot_l * reservoir.w
    }
}

#[inline]
fn random_f64(seed: &mut u32) -> f64 {
    *seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    (*seed >> 8) as f64 / 16777216.0
}

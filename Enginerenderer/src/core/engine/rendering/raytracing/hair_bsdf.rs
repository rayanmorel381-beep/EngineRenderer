use std::f64::consts::PI;
use super::math::Vec3;

pub const HAIR_COMPONENTS: usize = 3;

#[derive(Debug, Clone, Copy)]
pub struct HairMaterial {
    pub absorption: Vec3,
    pub roughness_longitudinal: f64,
    pub roughness_azimuthal: f64,
    pub scale_tilt_deg: f64,
    pub ior: f64,
    pub melanin: f64,
    pub melanin_redness: f64,
}

impl Default for HairMaterial {
    fn default() -> Self {
        Self {
            absorption: Vec3::new(0.06, 0.10, 0.20),
            roughness_longitudinal: 0.1,
            roughness_azimuthal: 0.2,
            scale_tilt_deg: 2.0,
            ior: 1.55,
            melanin: 0.3,
            melanin_redness: 0.5,
        }
    }
}

impl HairMaterial {
    pub fn blonde() -> Self {
        Self { melanin: 0.05, melanin_redness: 0.4, absorption: Vec3::new(0.014, 0.026, 0.06), ..Self::default() }
    }

    pub fn brunette() -> Self {
        Self { melanin: 0.4, melanin_redness: 0.55, absorption: Vec3::new(0.09, 0.15, 0.30), ..Self::default() }
    }

    pub fn black() -> Self {
        Self { melanin: 1.0, melanin_redness: 0.5, absorption: Vec3::new(0.60, 0.70, 0.85), ..Self::default() }
    }

    pub fn effective_absorption(&self) -> Vec3 {
        let mu_a = self.melanin;
        let mu_e = self.melanin * self.melanin_redness;
        let eumelanin = Vec3::new(0.419, 0.697, 1.372) * mu_a;
        let pheomelanin = Vec3::new(0.187, 0.4, 1.050) * mu_e;
        (eumelanin + pheomelanin) * 0.5 + self.absorption * 0.5
    }
}

pub struct MarschnerBsdf {
    pub material: HairMaterial,
}

impl MarschnerBsdf {
    pub fn new(material: HairMaterial) -> Self {
        Self { material }
    }

    pub fn evaluate(
        &self,
        wo: Vec3,
        wi: Vec3,
        hair_tangent: Vec3,
    ) -> Vec3 {
        let sin_theta_o = wo.dot(hair_tangent).clamp(-1.0, 1.0);
        let cos_theta_o = (1.0 - sin_theta_o * sin_theta_o).max(0.0).sqrt();
        let sin_theta_i = wi.dot(hair_tangent).clamp(-1.0, 1.0);
        let cos_theta_i = (1.0 - sin_theta_i * sin_theta_i).max(0.0).sqrt();

        let phi_o = azimuthal_angle(wo, hair_tangent);
        let phi_i = azimuthal_angle(wi, hair_tangent);
        let dphi = phi_i - phi_o;

        let alpha_r = (-self.material.scale_tilt_deg).to_radians();
        let alpha_tt = alpha_r * 0.5;
        let alpha_trt = -alpha_r * 1.5;

        let sin_theta_t = sin_theta_o / self.material.ior;
        let cos_theta_t = (1.0 - sin_theta_t * sin_theta_t).max(0.0).sqrt();
        let h_r = cos_theta_o * 0.0;
        let h_tt = 0.0_f64;
        let h_trt = 0.0_f64;
        let _ = (h_r, h_tt, h_trt, cos_theta_t);

        let sigma_a = self.material.effective_absorption();
        let l = 1.0 / cos_theta_t.max(0.01);
        let transmittance = Vec3::new(
            (-sigma_a.x * l).exp(),
            (-sigma_a.y * l).exp(),
            (-sigma_a.z * l).exp(),
        );

        let beta_n = self.material.roughness_longitudinal;
        let beta_m = self.material.roughness_azimuthal;

        let alphas = [alpha_r, alpha_tt, alpha_trt];
        let beta_ns = [beta_n, beta_n * 0.5, beta_n * 2.0];
        let m_lobes: [f64; HAIR_COMPONENTS] = std::array::from_fn(|p| {
            longitudinal_m(beta_ns[p], sin_theta_i, sin_theta_o, cos_theta_i, cos_theta_o, alphas[p])
        });

        let fr = fresnel_dielectric(cos_theta_o, self.material.ior);
        let ft = 1.0 - fr;

        let n_lobes: [Vec3; HAIR_COMPONENTS] = [
            Vec3::splat(azimuthal_n_r(beta_m, dphi)),
            azimuthal_n_tt(beta_m, dphi, self.material.ior, sigma_a),
            azimuthal_n_trt(beta_m, dphi, self.material.ior, sigma_a, transmittance),
        ];
        let weights: [f64; HAIR_COMPONENTS] = [fr, ft * ft, fr * ft * ft];

        let mut result = Vec3::ZERO;
        for p in 0..HAIR_COMPONENTS {
            result += n_lobes[p] * (m_lobes[p] * weights[p]);
        }

        result * (1.0 / cos_theta_i.max(0.01))
    }

    pub fn pdf(&self, wo: Vec3, wi: Vec3, hair_tangent: Vec3) -> f64 {
        let sin_theta_i = wi.dot(hair_tangent).clamp(-1.0, 1.0);
        let cos_theta_i = (1.0 - sin_theta_i * sin_theta_i).max(0.0).sqrt();
        let sin_theta_o = wo.dot(hair_tangent).clamp(-1.0, 1.0);
        let cos_theta_o = (1.0 - sin_theta_o * sin_theta_o).max(0.0).sqrt();
        let beta_n = self.material.roughness_longitudinal;
        let alpha_r = (-self.material.scale_tilt_deg).to_radians();
        let m_r = longitudinal_m(beta_n, sin_theta_i, sin_theta_o, cos_theta_i, cos_theta_o, alpha_r);
        (m_r / (2.0 * PI)).max(0.0)
    }
}

fn longitudinal_m(
    beta_n: f64,
    sin_theta_i: f64,
    sin_theta_o: f64,
    cos_theta_i: f64,
    cos_theta_o: f64,
    alpha: f64,
) -> f64 {
    let v = beta_n * beta_n;
    let sin_alpha = alpha.sin();
    let cos_alpha = alpha.cos();
    let sin_ti = sin_theta_i * cos_alpha + cos_theta_i * sin_alpha;
    let cos_to = cos_theta_o.abs();
    if v < 0.1 {
        let a = (-sin_theta_i * sin_ti) / v;
        let b = cos_theta_o * cos_to / v;
        a.exp() * b.cosh().ln_1p().exp() / (2.0 * v * (2.0 * PI * v).sqrt())
    } else {
        let exp_term = (-(sin_theta_o + sin_ti).powi(2) / (2.0 * v)).exp();
        exp_term / (2.0 * PI * v).sqrt() * (cos_theta_i.abs() * cos_to).sqrt().max(1e-9)
    }
}

fn azimuthal_n_r(beta_m: f64, dphi: f64) -> f64 {
    let v = beta_m * beta_m * 0.25;
    wrapped_cauchy(dphi - PI, v)
}

fn azimuthal_n_tt(beta_m: f64, dphi: f64, eta: f64, _sigma_a: Vec3) -> Vec3 {
    let v = beta_m * beta_m * 0.0625;
    let c = (1.0 / eta).asin();
    let phi_tt = PI + 2.0 * (2.0 * c / PI - 1.0).clamp(-1.0, 1.0).asin();
    let w = wrapped_cauchy(dphi - phi_tt, v);
    Vec3::splat(w)
}

fn azimuthal_n_trt(beta_m: f64, dphi: f64, eta: f64, _sigma_a: Vec3, transmittance: Vec3) -> Vec3 {
    let v = beta_m * beta_m;
    let c = (1.0 / eta).asin();
    let phi_trt = 4.0 * c - PI;
    let w = wrapped_cauchy(dphi - phi_trt, v);
    transmittance * transmittance * w
}

fn wrapped_cauchy(phi: f64, v: f64) -> f64 {
    if v < 0.001 {
        let d = phi.rem_euclid(2.0 * PI) - PI;
        return 1.0 / (d * d / v + 1.0) / (PI * v);
    }
    let mut sum = 0.0_f64;
    for k in -4i32..=4 {
        let p = phi - 2.0 * PI * k as f64;
        sum += (-(p * p) / (2.0 * v)).exp();
    }
    (sum / (2.0 * PI * v).sqrt()).max(0.0)
}

fn azimuthal_angle(v: Vec3, tangent: Vec3) -> f64 {
    let b1 = if tangent.x.abs() > tangent.z.abs() {
        Vec3::new(-tangent.y, tangent.x, 0.0).normalize()
    } else {
        Vec3::new(0.0, -tangent.z, tangent.y).normalize()
    };
    let b2 = tangent.cross(b1).normalize();
    v.dot(b1).atan2(v.dot(b2))
}

fn fresnel_dielectric(cos_i: f64, eta: f64) -> f64 {
    let sin2_t = (1.0 - cos_i * cos_i) / (eta * eta);
    if sin2_t >= 1.0 { return 1.0; }
    let cos_t = (1.0 - sin2_t).sqrt();
    let rs = (cos_i - eta * cos_t) / (cos_i + eta * cos_t);
    let rp = (eta * cos_i - cos_t) / (eta * cos_i + cos_t);
    (rs * rs + rp * rp) * 0.5
}

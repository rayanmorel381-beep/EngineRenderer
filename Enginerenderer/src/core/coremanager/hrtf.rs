pub struct HrtfProcessor {
    pub head_radius: f64,
    pub sample_rate: u32,
}

impl HrtfProcessor {
    pub fn new(head_radius: f64, sample_rate: u32) -> Self {
        Self {
            head_radius,
            sample_rate,
        }
    }

    pub fn process(&self, mono: &[f32], azimuth: f64, elevation: f64) -> Vec<[f32; 2]> {
        let speed_of_sound = 343.0_f64;
        let itd_max = self.head_radius / speed_of_sound;
        let itd = itd_max * azimuth.to_radians().sin() * elevation.to_radians().cos();
        let delay_samples = (itd * self.sample_rate as f64).round() as usize;

        let az_rad = azimuth.to_radians();
        let el_rad = elevation.to_radians();
        let ild_db = 6.0 * az_rad.sin() * el_rad.cos().abs();
        let gain_l = 10.0_f64.powf(-ild_db / 20.0) as f32;
        let gain_r = 10.0_f64.powf(ild_db / 20.0) as f32;

        let n = mono.len();
        let mut out = vec![[0.0_f32; 2]; n];
        for (i, sample) in out.iter_mut().enumerate().take(n) {
            let left_idx = if i >= delay_samples {
                i - delay_samples
            } else {
                i
            };
            let right_idx = if i + delay_samples < n {
                i + delay_samples
            } else {
                n - 1
            };
            sample[0] = mono[left_idx] * gain_l;
            sample[1] = mono[right_idx] * gain_r;
        }
        out
    }
}

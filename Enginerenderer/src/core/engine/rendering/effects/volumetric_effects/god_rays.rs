
use crate::core::engine::rendering::raytracing::Vec3;

pub struct GodRays {
    pub num_samples: u32,
    pub density: f64,
    pub weight: f64,
    pub decay: f64,
    pub exposure: f64,
}

impl GodRays {
    pub fn apply_to_buffer(
        &self,
        pixels: &mut [Vec3],
        width: usize,
        height: usize,
        light_screen_x: f64,
        light_screen_y: f64,
    ) {
        // Snapshot the original buffer so we march on a frozen copy.
        let snapshot: Vec<Vec3> = pixels.to_vec();

        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let tex_x = x as f64 / width as f64;
                let tex_y = y as f64 / height as f64;

                let delta_x =
                    (tex_x - light_screen_x) * self.density / self.num_samples as f64;
                let delta_y =
                    (tex_y - light_screen_y) * self.density / self.num_samples as f64;

                let mut sample_x = tex_x;
                let mut sample_y = tex_y;
                let mut color = Vec3::ZERO;
                let mut illumination_decay = 1.0;

                for _ in 0..self.num_samples {
                    sample_x -= delta_x;
                    sample_y -= delta_y;

                    let sx = (sample_x * width as f64)
                        .round()
                        .max(0.0)
                        .min(width as f64 - 1.0) as usize;
                    let sy = (sample_y * height as f64)
                        .round()
                        .max(0.0)
                        .min(height as f64 - 1.0) as usize;

                    let sampled = snapshot[sy * width + sx];
                    color += sampled * illumination_decay * self.weight;
                    illumination_decay *= self.decay;
                }

                pixels[idx] += color * self.exposure;
            }
        }
    }
}

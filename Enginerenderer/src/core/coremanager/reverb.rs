pub struct RoomConfig {
    pub width: f64,
    pub height: f64,
    pub depth: f64,
    pub absorption: f64,
}

pub struct ReverbProcessor {
    pub room: RoomConfig,
    pub sample_rate: u32,
}

impl ReverbProcessor {
    pub fn new(room: RoomConfig, sample_rate: u32) -> Self {
        Self { room, sample_rate }
    }

    pub fn process(&self, mono: &[f32], source: [f64; 3], listener: [f64; 3]) -> Vec<[f32; 2]> {
        let speed_of_sound = 343.0_f64;
        let n = mono.len();
        let mut out = vec![[0.0_f32; 2]; n];

        let max_order = 2_i32;
        for ix in -max_order..=max_order {
            for iy in -max_order..=max_order {
                for iz in -max_order..=max_order {
                    let image = [
                        if ix % 2 == 0 { source[0] + ix as f64 * self.room.width } else { ix as f64 * self.room.width - source[0] },
                        if iy % 2 == 0 { source[1] + iy as f64 * self.room.height } else { iy as f64 * self.room.height - source[1] },
                        if iz % 2 == 0 { source[2] + iz as f64 * self.room.depth } else { iz as f64 * self.room.depth - source[2] },
                    ];
                    let dx = image[0] - listener[0];
                    let dy = image[1] - listener[1];
                    let dz = image[2] - listener[2];
                    let dist = (dx*dx + dy*dy + dz*dz).sqrt().max(0.01);
                    let delay_s = dist / speed_of_sound;
                    let delay_samples = (delay_s * self.sample_rate as f64).round() as usize;
                    let reflections = (ix.abs() + iy.abs() + iz.abs()) as u32;
                    let attenuation = ((1.0 - self.room.absorption).powi(reflections as i32) / dist) as f32;
                    let az = dx.atan2(dz) as f32;
                    let pan_l = (0.5 - az * 0.15).clamp(0.0, 1.0);
                    let pan_r = (0.5 + az * 0.15).clamp(0.0, 1.0);
                    for (i, sample) in out.iter_mut().enumerate().take(n) {
                        let src_idx = if i >= delay_samples { i - delay_samples } else { continue };
                        let s = mono[src_idx] * attenuation;
                        sample[0] += s * pan_l;
                        sample[1] += s * pan_r;
                    }
                }
            }
        }
        out
    }
}

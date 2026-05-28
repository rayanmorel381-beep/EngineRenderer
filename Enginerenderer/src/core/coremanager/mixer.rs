use std::io::{self, Write};

pub struct AudioChannel {
    pub buffer: Vec<f32>,
    pub gain: f64,
    pub pan: f64,
}

impl AudioChannel {
    pub fn new(buffer: Vec<f32>, gain: f64, pan: f64) -> Self {
        Self { buffer, gain, pan: pan.clamp(-1.0, 1.0) }
    }
}

pub struct AudioMixer {
    pub channels: Vec<AudioChannel>,
    pub sample_rate: u32,
}

impl AudioMixer {
    pub fn new(sample_rate: u32) -> Self {
        Self { channels: Vec::new(), sample_rate }
    }

    pub fn add_channel(&mut self, channel: AudioChannel) {
        self.channels.push(channel);
    }

    pub fn mix_to_stereo(&self) -> Vec<[f32; 2]> {
        let max_len = self.channels.iter().map(|c| c.buffer.len()).max().unwrap_or(0);
        let mut out = vec![[0.0_f32; 2]; max_len];
        for ch in &self.channels {
            let gain = ch.gain as f32;
            let pan = ch.pan as f32;
            let gain_l = gain * (1.0 - pan.max(0.0));
            let gain_r = gain * (1.0 + pan.min(0.0));
            for (i, &s) in ch.buffer.iter().enumerate() {
                out[i][0] += s * gain_l;
                out[i][1] += s * gain_r;
            }
        }
        out
    }

    pub fn write_pcm_16bit<W: Write>(&self, writer: &mut W) -> io::Result<()> {
        let stereo = self.mix_to_stereo();
        for frame in &stereo {
            for &sample in frame {
                let clamped = sample.clamp(-1.0, 1.0);
                let pcm = (clamped * i16::MAX as f32) as i16;
                writer.write_all(&pcm.to_le_bytes())?;
            }
        }
        Ok(())
    }
}

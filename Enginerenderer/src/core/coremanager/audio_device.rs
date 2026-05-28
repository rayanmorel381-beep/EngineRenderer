use std::io::Write;

pub const AUDIO_RING_CAPACITY: usize = 8192;

pub struct AudioDevice {
    pub sample_rate: u32,
    pub buffer_size: usize,
    ring: Vec<[f32; 2]>,
    write_head: usize,
    read_head: usize,
    frames_written: u64,
}

impl AudioDevice {
    pub fn new(sample_rate: u32, buffer_size: usize) -> Self {
        let capacity = AUDIO_RING_CAPACITY.max(buffer_size * 4);
        Self {
            sample_rate,
            buffer_size,
            ring: vec![[0.0; 2]; capacity],
            write_head: 0,
            read_head: 0,
            frames_written: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.ring.len()
    }

    pub fn available_frames(&self) -> usize {
        let cap = self.ring.len();
        (self.write_head + cap - self.read_head) % cap
    }

    pub fn push_frame(&mut self, frame: [f32; 2]) {
        self.ring[self.write_head] = frame;
        self.write_head = (self.write_head + 1) % self.ring.len();
        self.frames_written += 1;
    }

    pub fn fill_from_stereo(&mut self, samples: &[[f32; 2]]) {
        for &s in samples {
            self.push_frame(s);
        }
    }

    pub fn drain_period(&mut self) -> Vec<[f32; 2]> {
        let count = self.available_frames().min(self.buffer_size);
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            out.push(self.ring[self.read_head]);
            self.read_head = (self.read_head + 1) % self.ring.len();
        }
        out
    }

    pub fn write_period_pcm16<W: Write>(&mut self, writer: &mut W) -> std::io::Result<usize> {
        let period = self.drain_period();
        let mut bytes_written = 0;
        for [l, r] in &period {
            let li = (*l * 32767.0).clamp(-32768.0, 32767.0) as i16;
            let ri = (*r * 32767.0).clamp(-32768.0, 32767.0) as i16;
            writer.write_all(&li.to_le_bytes())?;
            writer.write_all(&ri.to_le_bytes())?;
            bytes_written += 4;
        }
        Ok(bytes_written)
    }

    pub fn latency_ms(&self) -> f64 {
        self.available_frames() as f64 / self.sample_rate.max(1) as f64 * 1000.0
    }

    pub fn frames_written(&self) -> u64 {
        self.frames_written
    }
}

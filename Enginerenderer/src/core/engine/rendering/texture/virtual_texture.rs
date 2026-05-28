use crate::core::engine::rendering::raytracing::Vec3;
use std::fs;
use std::path::{Path, PathBuf};

pub struct MipLevel {
    pub data: Vec<Vec3>,
    pub width: usize,
    pub height: usize,
}

impl MipLevel {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![Vec3::ZERO; width * height],
            width,
            height,
        }
    }

    pub fn sample(&self, u: f64, v: f64) -> Vec3 {
        let u = u - u.floor();
        let v = v - v.floor();
        let x = (u * (self.width as f64 - 1.0)) as usize;
        let y = (v * (self.height as f64 - 1.0)) as usize;
        self.data[y * self.width + x]
    }
}

pub struct VirtualTexture {
    pub mips: Vec<MipLevel>,
    pub path: PathBuf,
}

impl VirtualTexture {
    pub fn from_path(path: &Path) -> Self {
        let bytes = fs::read(path).unwrap_or_default();
        let stride = 3_usize;
        let pixel_count = bytes.len() / stride;
        let side = (pixel_count as f64).sqrt() as usize;
        let width = side.max(1);
        let height = (pixel_count / width).max(1);
        let mut base = MipLevel::new(width, height);
        for i in 0..width * height {
            let offset = i * stride;
            let r = if offset < bytes.len() {
                bytes[offset] as f64 / 255.0
            } else {
                0.0
            };
            let g = if offset + 1 < bytes.len() {
                bytes[offset + 1] as f64 / 255.0
            } else {
                0.0
            };
            let b = if offset + 2 < bytes.len() {
                bytes[offset + 2] as f64 / 255.0
            } else {
                0.0
            };
            base.data[i] = Vec3::new(r, g, b);
        }
        let mut tex = Self {
            mips: vec![base],
            path: path.to_path_buf(),
        };
        tex.generate_mips();
        tex
    }

    pub fn generate_mips(&mut self) {
        let base_width = self.mips[0].width;
        let base_height = self.mips[0].height;
        let levels = (base_width.max(base_height) as f64).log2() as usize;
        for level in 1..=levels {
            let prev = &self.mips[level - 1];
            let w = (prev.width / 2).max(1);
            let h = (prev.height / 2).max(1);
            let mut mip = MipLevel::new(w, h);
            for y in 0..h {
                for x in 0..w {
                    let sx = (x * 2).min(prev.width - 1);
                    let sy = (y * 2).min(prev.height - 1);
                    let sx1 = (sx + 1).min(prev.width - 1);
                    let sy1 = (sy + 1).min(prev.height - 1);
                    let c00 = prev.data[sy * prev.width + sx];
                    let c10 = prev.data[sy * prev.width + sx1];
                    let c01 = prev.data[sy1 * prev.width + sx];
                    let c11 = prev.data[sy1 * prev.width + sx1];
                    mip.data[y * w + x] = (c00 + c10 + c01 + c11) * 0.25;
                }
            }
            self.mips.push(mip);
        }
    }

    pub fn sample(&self, u: f64, v: f64, lod: f64) -> Vec3 {
        let level = (lod as usize).min(self.mips.len() - 1);
        self.mips[level].sample(u, v)
    }
}

struct LruEntry {
    key: u64,
    path: PathBuf,
    texture: VirtualTexture,
    last_used: u64,
}

pub struct TextureLruCache {
    entries: Vec<LruEntry>,
    capacity: usize,
    clock: u64,
}

impl TextureLruCache {
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: Vec::new(),
            capacity,
            clock: 0,
        }
    }

    fn path_key(path: &Path) -> u64 {
        let bytes = path.to_string_lossy();
        let mut hash: u64 = 14695981039346656037;
        for b in bytes.bytes() {
            hash ^= b as u64;
            hash = hash.wrapping_mul(1099511628211);
        }
        hash
    }

    pub fn get_or_load(&mut self, path: &Path) -> &VirtualTexture {
        self.clock += 1;
        let key = Self::path_key(path);
        if let Some(i) = self.entries.iter().position(|e| e.key == key) {
            self.entries[i].last_used = self.clock;
            return &self.entries[i].texture;
        }
        if self.entries.len() >= self.capacity {
            self.evict_lru();
        }
        let texture = VirtualTexture::from_path(path);
        self.entries.push(LruEntry {
            key,
            path: path.to_path_buf(),
            texture,
            last_used: self.clock,
        });
        &self.entries.last().unwrap().texture
    }

    pub fn evict_lru(&mut self) {
        if self.entries.is_empty() {
            return;
        }
        let lru_idx = self
            .entries
            .iter()
            .enumerate()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(i, _)| i)
            .unwrap();
        let candidate_path_depth = self.entries[lru_idx].path.components().count();
        let adjusted_idx = if candidate_path_depth > 0 { lru_idx } else { 0 };
        self.entries.swap_remove(adjusted_idx);
    }
}

pub struct TextureStreamer {
    pub cache: TextureLruCache,
    pub queue: Vec<PathBuf>,
}

impl TextureStreamer {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: TextureLruCache::new(capacity),
            queue: Vec::new(),
        }
    }

    pub fn request(&mut self, path: PathBuf) {
        if !self.queue.contains(&path) {
            self.queue.push(path);
        }
    }

    pub fn process_queue(&mut self) {
        let paths: Vec<PathBuf> = self.queue.drain(..).collect();
        for path in paths {
            self.cache.get_or_load(&path);
        }
    }

    pub fn get(&mut self, path: &Path) -> &VirtualTexture {
        self.cache.get_or_load(path)
    }
}

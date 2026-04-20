use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread;

use crate::core::engine::acces_hardware::{
    self, CpuProfile, HardwareCapabilities, NativeHardwareBackend,
    precise_timestamp_ns, elapsed_ms, gpu_dispatch_tiles,
    alloc_dma_framebuffer, DmaFramebuffer,
};
use crate::core::engine::acces_hardware::cpu::{
    detect_core_frequencies, thread_affinity_mask, CoreSnapshot,
};

// ─── Hardware snapshot (cached once per scheduler) ─────────────────────

#[derive(Debug, Clone)]
pub struct HwContext {
    pub caps: HardwareCapabilities,
    pub cpu: CpuProfile,
    pub core_freqs: Vec<CoreSnapshot>,
}

impl HwContext {
    pub fn detect() -> Self {
        let backend = NativeHardwareBackend::detect();
        Self::from_backend(&backend)
    }

    pub fn from_backend(backend: &NativeHardwareBackend) -> Self {
        let caps = backend.hw_caps().clone();
        let cpu = backend.cpu_profile().clone();
        let core_freqs = detect_core_frequencies();
        Self { caps, cpu, core_freqs }
    }

    pub fn fastest_cores(&self, n: usize) -> Vec<u32> {
        let mut sorted: Vec<_> = self.core_freqs.clone();
        sorted.sort_by(|a, b| b.frequency_hz.cmp(&a.frequency_hz));
        sorted.iter().take(n).map(|c| c.core_id).collect()
    }

    pub fn max_pixel_bytes(&self, input_pixels: usize) -> u64 {
        self.caps.max_framebuffer_bytes_for_input(input_pixels)
    }

    pub fn simd_tile_width(&self) -> usize {
        self.cpu.optimal_tile_width()
    }

    pub fn l2_tile_pixels(&self) -> usize {
        let l2_bytes = (self.cpu.l2_cache_kb as usize) * 1024;
        // Each pixel = Vec3 = 3×f64 = 24 bytes. Use 75% of L2.
        let usable = l2_bytes * 3 / 4;
        (usable / 24).max(64)
    }

    pub fn simd_tag(&self) -> &'static str {
        let s = &self.cpu.simd_features;
        if s.avx512f { "AVX-512" }
        else if s.avx2 { "AVX2" }
        else if s.avx { "AVX" }
        else if s.fma { "FMA" }
        else if s.sse4_2 { "SSE4.2" }
        else if s.sse2 { "SSE2" }
        else if s.neon { "NEON" }
        else { "scalar" }
    }
}

// ─── Tile ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Tile {
    pub index: usize,
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

// ─── Per-worker statistics ─────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct WorkerStats {
    pub worker_id: usize,
    pub core_id: u32,
    pub tiles_rendered: usize,
    pub pixels_rendered: usize,
    pub total_ns: u64,
    pub affinity_mask: usize,
}

// ─── Scheduler result ──────────────────────────────────────────────────

#[derive(Debug)]
pub struct SchedulerReport {
    pub worker_stats: Vec<WorkerStats>,
    pub total_tiles: usize,
    pub total_pixels: usize,
    pub dispatch_ns: u64,
    pub gpu_dispatched: bool,
    pub dma_allocated: bool,
    pub simd_tag: &'static str,
}

impl SchedulerReport {
    pub fn dispatch_ms(&self) -> f64 {
        elapsed_ms(0, self.dispatch_ns)
    }

    pub fn log_summary(&self) {
        crate::runtime_log!(
            "scheduler: {} tiles, {} pixels, {:.1}ms, {} workers, simd={}, gpu={}, dma={}",
            self.total_tiles, self.total_pixels,
            self.dispatch_ms(),
            self.worker_stats.len(),
            self.simd_tag,
            self.gpu_dispatched,
            self.dma_allocated,
        );
        for w in &self.worker_stats {
            let ms = elapsed_ms(0, w.total_ns);
            crate::runtime_log!(
                "  worker-{}: core={} tiles={} pixels={} {:.1}ms affinity=0x{:x}",
                w.worker_id, w.core_id, w.tiles_rendered, w.pixels_rendered,
                ms, w.affinity_mask,
            );
        }
    }
}

// ─── Adaptive tile scheduler ───────────────────────────────────────────

#[derive(Debug)]
pub struct TileScheduler {
    total_tiles: usize,
    tile_width: usize,
    tile_height: usize,
    image_width: usize,
    image_height: usize,
    worker_count: usize,
    hw: HwContext,
    fastest_cores: Vec<u32>,
    dma_fb: Option<DmaFramebuffer>,
}

impl TileScheduler {
    pub fn new(image_width: usize, image_height: usize, hint_threads: usize) -> Self {
        let backend = NativeHardwareBackend::detect();
        Self::new_with_backend(image_width, image_height, hint_threads, &backend)
    }

    pub fn new_with_backend(
        image_width: usize,
        image_height: usize,
        hint_threads: usize,
        backend: &NativeHardwareBackend,
    ) -> Self {
        let hw = HwContext::from_backend(backend);

        // ── Memory guard: refuse resolutions that exceed available RAM ──
        let pixel_bytes = (image_width * image_height * 24) as u64; // 3×f64
        let max_bytes = hw.max_pixel_bytes(image_width.saturating_mul(image_height));
        if pixel_bytes > max_bytes {
            crate::runtime_log!(
                "scheduler: WARNING resolution {}×{} needs {}MB but only {}MB available",
                image_width, image_height,
                pixel_bytes / (1024 * 1024),
                max_bytes / (1024 * 1024),
            );
        }

        let worker_count = effective_workers(&hw, image_width, image_height, hint_threads);
        let fastest_cores = hw.fastest_cores(worker_count);
        let (tile_width, tile_height) = adaptive_tile_size(&hw, image_width, image_height, worker_count);

        let cols = image_width.div_ceil(tile_width.max(1));
        let rows = image_height.div_ceil(tile_height.max(1));

        // ── Attempt DMA-coherent framebuffer allocation ─────────────────
        let dma_fb = alloc_dma_framebuffer(image_width, image_height);
        if let Some(ref fb) = dma_fb {
            crate::runtime_log!(
                "scheduler: DMA framebuffer allocated ({}×{}, {}B, phys=0x{:x})",
                image_width, image_height,
                fb.byte_len(),
                fb.virt_addr(),
            );
        }

        crate::runtime_log!(
            "scheduler: {}×{} tiles={}×{}={} workers={}/{} simd={} l2_tile={}px fastest_cores={:?}",
            image_width, image_height,
            cols, rows, cols * rows,
            worker_count, hw.caps.logical_cores,
            hw.simd_tag(),
            hw.l2_tile_pixels(),
            &fastest_cores,
        );

        Self {
            total_tiles: cols * rows,
            tile_width,
            tile_height,
            image_width,
            image_height,
            worker_count,
            hw,
            fastest_cores,
            dma_fb,
        }
    }

    pub fn total_tiles(&self) -> usize {
        self.total_tiles
    }

    pub fn worker_count(&self) -> usize {
        self.worker_count
    }

    pub fn hw(&self) -> &HwContext {
        &self.hw
    }

    pub fn has_dma(&self) -> bool {
        self.dma_fb.is_some()
    }

    pub fn dma_ptr(&self) -> Option<*mut u8> {
        self.dma_fb.as_ref().map(|fb| fb.as_ptr())
    }

    pub fn tile_at(&self, index: usize) -> Tile {
        let cols = self.image_width.div_ceil(self.tile_width.max(1));
        let tile_row = index / cols;
        let tile_col = index % cols;
        let x = tile_col * self.tile_width;
        let y = tile_row * self.tile_height;
        Tile {
            index,
            x,
            y,
            width: self.tile_width.min(self.image_width.saturating_sub(x)),
            height: self.tile_height.min(self.image_height.saturating_sub(y)),
        }
    }

    pub fn dispatch<F, T>(&self, work_fn: F) -> (Vec<(usize, Vec<T>)>, SchedulerReport)
    where
        F: Fn(Tile) -> Vec<T> + Sync,
        T: Send,
    {
        let dispatch_start = precise_timestamp_ns();

        // ── GPU dispatch tracking ───────────────────────────────────────
        let workgroup = self.tile_width.max(1) * self.tile_height.max(1);
        let gpu_dispatched = gpu_dispatch_tiles(self.total_tiles as u32, workgroup as u32) > 0;

        let next_tile = AtomicUsize::new(0);
        let abort = AtomicBool::new(false);
        let total = self.total_tiles;

        // Per-worker atomic counters for tile/pixel stats
        let worker_tile_counts: Vec<AtomicUsize> = (0..self.worker_count)
            .map(|_| AtomicUsize::new(0))
            .collect();
        let worker_pixel_counts: Vec<AtomicUsize> = (0..self.worker_count)
            .map(|_| AtomicUsize::new(0))
            .collect();
        let worker_ns: Vec<AtomicU64> = (0..self.worker_count)
            .map(|_| AtomicU64::new(0))
            .collect();

        let mut all_results: Vec<(usize, Vec<T>)> = Vec::with_capacity(total);

        thread::scope(|scope| {
            let mut handles = Vec::with_capacity(self.worker_count);

            for worker_id in 0..self.worker_count {
                let next_ref = &next_tile;
                let abort_ref = &abort;
                let work_ref = &work_fn;
                let tc = &worker_tile_counts[worker_id];
                let pc = &worker_pixel_counts[worker_id];
                let ns = &worker_ns[worker_id];
                let core_id = self.fastest_cores.get(worker_id).copied()
                    .unwrap_or(worker_id as u32);

                let handle_result = thread::Builder::new()
                    .name(format!("tile-worker-{worker_id}"))
                    .stack_size(8 * 1024 * 1024)
                    .spawn_scoped(scope, move || {
                        // Pin to fastest available physical core
                        acces_hardware::pin_thread_to_core(core_id as usize);
                        let worker_start = precise_timestamp_ns();

                        let mut local = Vec::new();
                        loop {
                            if abort_ref.load(Ordering::Relaxed) {
                                break;
                            }
                            let idx = next_ref.fetch_add(1, Ordering::Relaxed);
                            if idx >= total {
                                break;
                            }
                            let tile = self.tile_at(idx);
                            let tile_pixels = tile.width * tile.height;
                            let pixels = work_ref(tile);
                            tc.fetch_add(1, Ordering::Relaxed);
                            pc.fetch_add(tile_pixels, Ordering::Relaxed);
                            local.push((idx, pixels));
                        }

                        let elapsed = precise_timestamp_ns().saturating_sub(worker_start);
                        ns.store(elapsed, Ordering::Relaxed);
                        local
                    });

                match handle_result {
                    Ok(handle) => handles.push(handle),
                    Err(_) => {
                        let worker_start = precise_timestamp_ns();
                        let mut local = Vec::new();
                        loop {
                            if abort_ref.load(Ordering::Relaxed) {
                                break;
                            }
                            let idx = next_ref.fetch_add(1, Ordering::Relaxed);
                            if idx >= total {
                                break;
                            }
                            let tile = self.tile_at(idx);
                            let tile_pixels = tile.width * tile.height;
                            let pixels = work_ref(tile);
                            tc.fetch_add(1, Ordering::Relaxed);
                            pc.fetch_add(tile_pixels, Ordering::Relaxed);
                            local.push((idx, pixels));
                        }
                        let elapsed = precise_timestamp_ns().saturating_sub(worker_start);
                        ns.store(elapsed, Ordering::Relaxed);
                        all_results.extend(local);
                    }
                }
            }

            for h in handles {
                match h.join() {
                    Ok(results) => all_results.extend(results),
                    Err(_) => abort.store(true, Ordering::Relaxed),
                }
            }
        });

        all_results.sort_by_key(|(idx, _)| *idx);

        let dispatch_elapsed = precise_timestamp_ns().saturating_sub(dispatch_start);

        // ── Build per-worker stats ──────────────────────────────────────
        let worker_stats: Vec<WorkerStats> = (0..self.worker_count)
            .map(|i| {
                let core_id = self.fastest_cores.get(i).copied().unwrap_or(i as u32);
                WorkerStats {
                    worker_id: i,
                    core_id,
                    tiles_rendered: worker_tile_counts[i].load(Ordering::Relaxed),
                    pixels_rendered: worker_pixel_counts[i].load(Ordering::Relaxed),
                    total_ns: worker_ns[i].load(Ordering::Relaxed),
                    affinity_mask: thread_affinity_mask(),
                }
            })
            .collect();

        let total_pixels = worker_stats.iter().map(|w| w.pixels_rendered).sum();

        let report = SchedulerReport {
            worker_stats,
            total_tiles: total,
            total_pixels,
            dispatch_ns: dispatch_elapsed,
            gpu_dispatched,
            dma_allocated: self.dma_fb.is_some(),
            simd_tag: self.hw.simd_tag(),
        };

        (all_results, report)
    }
}

fn effective_workers(hw: &HwContext, w: usize, h: usize, hint: usize) -> usize {
    let cpus = (hw.caps.logical_cores as usize).max(1);
    let max_by_pixels = (w * h).div_ceil(3_000).max(1);
    let max_by_rows = h.div_ceil(8).max(1);
    hint.min(cpus).min(max_by_pixels).min(max_by_rows).max(1)
}

fn adaptive_tile_size(hw: &HwContext, w: usize, h: usize, workers: usize) -> (usize, usize) {
    let simd_w = hw.simd_tile_width();
    let l2_pixels = hw.l2_tile_pixels();

    // Tile width: aligned to SIMD, at most half image width for ≥2 columns
    let tile_w = if w > simd_w * 4 {
        // Multiple columns, each SIMD-aligned
        let cols = (workers * 2).max(2);
        let raw = w.div_ceil(cols);
        // Round up to SIMD boundary
        raw.div_ceil(simd_w).clamp(simd_w, w)
    } else {
        // Small image: full width, SIMD-aligned
        w.div_ceil(simd_w).max(simd_w)
    };

    // Tile height: fit in L2, with enough tiles for 8× oversubscription
    let target_tiles = (workers * 8).max(4);
    let max_h_by_l2 = l2_pixels.div_ceil(tile_w.max(1));
    let max_h_by_balance = h.div_ceil(target_tiles);
    let tile_h = max_h_by_l2.min(max_h_by_balance).clamp(4, 128);

    (tile_w, tile_h)
}

//! Tests for the rendering LOD manager and tile scheduler.

use enginerenderer::api::engine::objects::*;
use enginerenderer::api::engine::rendering::*;
use enginerenderer::api::objects::primitives::Vec3;

// ── Rendering LodManager ────────────────────────────────────────────────

#[test]
fn rendering_lod_select() {
    let lod = RenderingLodManager::default()
        .with_thresholds(LodThresholds::default())
        .with_hysteresis(12.0);
    let sel = lod.select(50.0, 200.0);
    assert!(sel.primary_samples > 0);
}

#[test]
fn rendering_lod_select_with_hysteresis() {
    let mut lod = RenderingLodManager::default()
        .with_thresholds(LodThresholds::default())
        .with_hysteresis(12.0);
    let sel = lod.select_with_hysteresis(0, 50.0, 200.0);
    assert!(sel.primary_samples > 0);
}

#[test]
fn rendering_lod_select_for_tier() {
    let lod = RenderingLodManager::default();
    let sel = lod.select_for_tier(LodTier::Ultra);
    assert!(sel.primary_samples > 0);
}

#[test]
fn rendering_lod_screen_space_error_and_refine() {
    let lod = RenderingLodManager::default();
    let sse = lod.screen_space_error(0.5, 100.0, 1080.0, 60.0_f64.to_radians());
    assert!(sse.is_finite());
    let _should_ref = lod.should_refine(0.5, 100.0, 1080.0, 60.0_f64.to_radians());
}

#[test]
fn rendering_lod_horizon_detail_for() {
    let lod = RenderingLodManager::default();
    let hd = lod.horizon_detail_for(
        Vec3::ZERO,
        Vec3::new(100.0, 0.0, 0.0),
        500.0,
    );
    assert!(hd.is_finite());
}

// ── TileScheduler ───────────────────────────────────────────────────────

#[test]
fn tile_scheduler_layout() {
    let sched = TileScheduler::new(1600, 900, 4);
    let total = sched.total_tiles();
    let workers = sched.worker_count();
    assert!(total > 0);
    assert!(workers > 0);
    let tile = sched.tile_at(0);
    assert!(tile.width > 0 && tile.height > 0);
}

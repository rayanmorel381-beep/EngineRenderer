//! Tests for internal coremanager systems: ConfigManager, LodManager,
//! ResourceTracker, NetworkManager, EventBus, PhysicsManager, Logger.

use enginerenderer::api::engine::objects::*;
use enginerenderer::api::engine::scenes::*;
use enginerenderer::api::engine::cameras::*;

// ── ConfigManager & ConfigPreset ────────────────────────────────────────

#[test]
fn config_manager_set_resolution_marks_dirty() {
    let config = EngineConfig::ultra_hd_cpu();
    let mut mgr = ConfigManager::new(config.clone());
    mgr.set_resolution(config.width, config.height);
    assert!(mgr.is_dirty());
    assert!(mgr.validate().is_ok());
    assert!(mgr.config().width > 0);
    mgr.config_mut().height = config.height;
    mgr.apply();
}

#[test]
fn config_manager_from_presets() {
    let ultra = ConfigManager::from_preset(ConfigPreset::UltraHd);
    let prod = ConfigManager::from_preset(ConfigPreset::Production);
    assert!(ultra.config().width > 0);
    assert!(prod.config().width > 0);
}

// ── CoreLodManager ──────────────────────────────────────────────────────

#[test]
fn core_lod_manager_select_and_horizon() {
    let lod = CoreLodManager::default();
    let selection = lod.select(10.0, 100.0);
    assert!(selection.primary_samples > 0);
    let horizon = lod.horizon_detail(500.0);
    assert!(horizon > 0.0);
}

// ── ResourceTracker ─────────────────────────────────────────────────────

#[test]
fn resource_tracker_records_outputs() {
    let mut tracker = ResourceTracker::new();
    tracker.record_output(std::path::PathBuf::from("/tmp/test_output.ppm"));
    assert_eq!(tracker.output_count(), 1);
    assert!(!tracker.outputs().is_empty());
    assert!(tracker.has_outputs());
}

// ── NetworkManager ──────────────────────────────────────────────────────

#[test]
fn network_manager_sync_scene() {
    let bodies = CelestialBodies::showcase();
    let graph = SceneGraph::from_bodies(&bodies);
    let mut mgr = NetworkManager::new(2);
    let snapshot = mgr.sync_scene(&graph, 0);
    assert_eq!(snapshot.frame_index, 0);
    assert!(snapshot.node_count > 0);
    assert!(snapshot.scene_radius > 0.0);
    assert!(snapshot.scene_origin.length() >= 0.0);
    assert!(mgr.remote_client_count() >= 1);
}

// ── EventBus ────────────────────────────────────────────────────────────

#[test]
fn event_bus_push_drain_history() {
    use enginerenderer::api::engine::scenes::EventBus;
    let mut bus = EventBus::default();
    bus.push(enginerenderer::api::engine::scenes::EngineEvent::FrameStarted {
        frame_index: 0,
        target_ms: 16.0,
    });
    assert!(bus.history_len() >= 1);
    let summary = bus.summarize_history();
    assert!(summary.last_frame_index == 0);
    let drained = bus.drain();
    assert!(!drained.is_empty());
}

// ── EventLog ────────────────────────────────────────────────────────────

#[test]
fn event_log_record_and_query() {
    let mut log = EventLog::new();
    let mut bus = EventBus::default();
    bus.push(enginerenderer::api::engine::scenes::EngineEvent::FrameStarted {
        frame_index: 0,
        target_ms: 16.0,
    });
    let snap = bus.summarize_history();
    log.record(snap);
    assert_eq!(log.len(), 1);
    assert!(!log.is_empty());
    assert!(log.latest().is_some());
    assert!(!log.snapshots().is_empty());
    let _total = log.total_pixels();
}

// ── PhysicsManager ──────────────────────────────────────────────────────

#[test]
fn physics_manager_invariants() {
    let bodies = CelestialBodies::showcase();
    let pm = PhysicsManager::from_bodies(&bodies);
    assert!(pm.total_kinetic_energy() >= 0.0);
    assert!(pm.total_momentum() >= 0.0);
    assert!(pm.average_orbital_radius() >= 0.0);
    assert!(pm.net_gravity_measure() >= 0.0);
    assert!(pm.stability_score() >= 0.0);
}

// ── EngineLogger ────────────────────────────────────────────────────────

#[test]
fn logger_tracks_messages() {
    let mut logger = EngineLogger::with_capacity(16);
    assert!(logger.is_empty());
    assert!(logger.latest_message().is_none());
    logger.info("hello".to_string());
    assert!(!logger.is_empty());
    assert!(logger.latest_message().is_some());
}

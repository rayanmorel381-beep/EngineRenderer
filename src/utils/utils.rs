

// =====================================================================
// Logger
// =====================================================================

#[derive(Debug, Clone)]
pub struct EngineLogger {
    messages: Vec<(LogLevel, String)>,
    warning_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Debug,
    Info,
    Warning,
}

impl EngineLogger {
    pub fn with_capacity(cap: usize) -> Self {
        Self {
            messages: Vec::with_capacity(cap),
            warning_count: 0,
        }
    }

    pub fn debug(&mut self, msg: String) {
        self.messages.push((LogLevel::Debug, msg));
    }

    pub fn info(&mut self, msg: String) {
        self.messages.push((LogLevel::Info, msg));
    }

    pub fn warning(&mut self, msg: String) {
        self.warning_count += 1;
        self.messages.push((LogLevel::Warning, msg));
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    pub fn latest_message(&self) -> Option<&str> {
        self.messages.last().map(|(_, msg)| msg.as_str())
    }
}

// =====================================================================
// Frame profiler & timing
// =====================================================================

use crate::core::engine::acces_hardware::timer::HwInstant;

#[derive(Debug, Clone)]
pub struct FrameProfile {
    frame_index: u64,
    start: HwInstant,
    simulation_done: Option<HwInstant>,
    scene_prepared: Option<HwInstant>,
}

impl FrameProfile {
    pub fn mark_simulation_complete(&mut self) {
        self.simulation_done = Some(HwInstant::now());
    }

    pub fn mark_scene_prepared(&mut self) {
        self.scene_prepared = Some(HwInstant::now());
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FrameSummary {
    pub frame_index: u64,
    pub total_frame_ms: u128,
    pub simulation_ms: u128,
    pub scene_prep_ms: u128,
    pub rendered_pixels: usize,
    pub scene_nodes: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct FrameProfiler;

impl FrameProfiler {
    pub fn begin_frame(&self, frame_index: u64) -> FrameProfile {
        FrameProfile {
            frame_index,
            start: HwInstant::now(),
            simulation_done: None,
            scene_prepared: None,
        }
    }

    pub fn finish_frame(
        &self,
        profile: FrameProfile,
        report: &RenderReport,
        scene_nodes: usize,
    ) -> FrameSummary {
        let now = HwInstant::now();
        let total = now.duration_since_ms(&profile.start);
        let sim_ms = profile
            .simulation_done
            .map(|t| t.duration_since_ms(&profile.start))
            .unwrap_or(0);
        let prep_ms = profile
            .scene_prepared
            .map(|t| {
                let base = profile.simulation_done.unwrap_or(profile.start);
                t.duration_since_ms(&base)
            })
            .unwrap_or(0);

        FrameSummary {
            frame_index: profile.frame_index,
            total_frame_ms: total,
            simulation_ms: sim_ms,
            scene_prep_ms: prep_ms,
            rendered_pixels: report.rendered_pixels,
            scene_nodes,
        }
    }
}

// =====================================================================
// Debug tools & overlay
// =====================================================================

#[derive(Debug, Clone)]
pub struct DebugOverlay {
    pub headline: String,
}

#[derive(Debug, Clone, Copy)]
pub struct DebugTools;

impl DebugTools {
    pub fn capture(
        &self,
        summary: &FrameSummary,
        report: &RenderReport,
        network_status: &str,
        audio_mix: AudioMix,
        event_summary: &EventSummary,
        warning_count: usize,
        total_momentum: f64,
        log_len: usize,
    ) -> DebugOverlay {
        DebugOverlay {
            headline: format!(
                "frame={} pixels={} ms={} net={} audio={:.2} warnings={} momentum={:.4} events={} logs={}",
                summary.frame_index,
                summary.rendered_pixels,
                summary.total_frame_ms,
                network_status,
                audio_mix.master_gain,
                warning_count,
                total_momentum,
                event_summary.last_frame_index,
                log_len,
            ),
        }
    }
}

// =====================================================================
// Serialization
// =====================================================================

#[derive(Debug, Clone, Copy)]
pub struct SerializationManager;

impl SerializationManager {
    pub fn serialize_overlay(&self, overlay: &DebugOverlay) -> String {
        format!(
            "frame={headline}\nwarnings={w}",
            headline = overlay.headline,
            w = overlay.headline.matches("warnings=").count(),
        )
    }

    pub fn serialize_validation_report(
        &self,
        summary: &FrameSummary,
        report: &RenderReport,
        overlay: &DebugOverlay,
    ) -> String {
        format!(
            "validation\nframe={}\npixels={}\nrender_ms={}\navg_luminance={:.9}\ntriangles={}\nobjects={}\noverlay={}\n",
            summary.frame_index,
            summary.rendered_pixels,
            report.duration_ms,
            report.average_luminance,
            report.triangle_count,
            report.object_count,
            overlay.headline,
        )
    }

    pub fn write_text_report(&self, path: &Path, content: &str) -> Result<(), Box<dyn Error>> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, content)?;
        Ok(())
    }
}

// =====================================================================
// Audio
// =====================================================================

#[derive(Debug, Clone, Copy)]
pub struct AudioMix {
    pub master_gain: f64,
    pub spatial_width: f64,
    pub reverb_send: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct AudioManager {
    base_gain: f64,
}

impl AudioManager {
    pub fn new(base_gain: f64) -> Self {
        Self {
            base_gain: base_gain.clamp(0.1, 2.0),
        }
    }

    pub fn mix_for_scene(
        &self,
        graph: &SceneGraph,
        camera_distance: f64,
        exposure_bias: f64,
    ) -> AudioMix {
        let luminous_factor = graph.luminous_node_count().max(1) as f64;
        let distance_factor = (camera_distance / (graph.scene_radius() + 1.0)).clamp(0.5, 2.0);

        AudioMix {
            master_gain: self.base_gain * (0.85 + luminous_factor * 0.03) / distance_factor,
            spatial_width: (0.55 + graph.node_count() as f64 * 0.04).clamp(0.55, 1.0),
            reverb_send: (0.12 + exposure_bias * 0.18).clamp(0.1, 0.4),
        }
    }
}

// =====================================================================
// Network
// =====================================================================

#[derive(Debug, Clone)]
pub struct NetworkSnapshot {
    pub checksum: u64,
    pub scene_radius: f64,
    pub scene_origin: Vec3,
}

#[derive(Debug, Clone)]
pub struct NetworkManager {
    remote_clients: usize,
    status: String,
}

impl NetworkManager {
    pub fn new(remote_clients: usize) -> Self {
        Self {
            remote_clients: remote_clients.max(1),
            status: "connected".to_string(),
        }
    }

    pub fn sync_scene(&self, graph: &SceneGraph, frame_index: u64) -> NetworkSnapshot {
        let mut hasher: u64 = frame_index;
        hasher = hasher.wrapping_mul(6_364_136_223_846_793_005);
        hasher = hasher.wrapping_add(graph.node_count() as u64);
        hasher ^= (graph.scene_radius() * 1e6) as u64;

        NetworkSnapshot {
            checksum: hasher,
            scene_radius: graph.scene_radius(),
            scene_origin: graph.focus_point(),
        }
    }

    pub fn status(&self) -> &str {
        &self.status
    }

    pub fn remote_client_count(&self) -> usize {
        self.remote_clients
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RenderSyncServer {
    capacity: usize,
}

impl RenderSyncServer {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity: capacity.max(1),
        }
    }

    pub fn publish(&self, _frame_index: u64, _snapshot: &NetworkSnapshot) -> usize {
        self.capacity
    }

    pub fn client_count(&self) -> usize {
        self.capacity
    }
}

// =====================================================================
// Input
// =====================================================================

#[derive(Debug, Clone, Copy)]
pub struct CinematicInput {
    pub orbit_bias: f64,
    pub exposure_nudge: f64,
    pub time_scale: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct InputManager {
    cinematic_mode: bool,
}

impl InputManager {
    pub fn new(cinematic_mode: bool) -> Self {
        Self { cinematic_mode }
    }

    pub fn sample_cinematic_input(&self, time: f64) -> CinematicInput {
        if !self.cinematic_mode {
            return CinematicInput {
                orbit_bias: 0.0,
                exposure_nudge: 0.0,
                time_scale: 1.0,
            };
        }

        CinematicInput {
            orbit_bias: (time * 0.7).sin() * 0.35,
            exposure_nudge: (time * 0.45).cos() * 0.04,
            time_scale: (1.0 + (time * 0.2).sin() * 0.08).clamp(0.92, 1.08),
        }
    }
}

// =====================================================================
// Camera
// =====================================================================

use crate::rendering::ray_tracing::Camera;

#[derive(Debug, Clone, Copy)]
pub struct CameraManager {
    focus_point: Vec3,
    orbit_radius: f64,
    height: f64,
    vertical_fov: f64,
}

impl CameraManager {
    pub fn cinematic_for_scene(focus_point: Vec3, scene_radius: f64) -> Self {
        let mut manager = Self {
            focus_point,
            orbit_radius: 10.0,
            height: 2.5,
            vertical_fov: 36.0,
        };
        manager.reframe(focus_point, scene_radius);
        manager
    }

    pub fn reframe(&mut self, focus_point: Vec3, scene_radius: f64) {
        let safe_radius = scene_radius.max(1.0);
        self.focus_point = focus_point;
        self.orbit_radius = (safe_radius * 2.35).clamp(8.0, 42.0);
        self.height = (safe_radius * 0.70).clamp(2.0, 12.0);
        self.vertical_fov = (34.0 + safe_radius * 0.55).clamp(32.0, 46.0);
    }

    pub fn build_camera(&self, aspect_ratio: f64, time: f64) -> Camera {
        let yaw = 0.25 + time * 0.45;
        let vertical_motion = (time * 0.8).sin() * 0.35;
        let origin = self.focus_point
            + Vec3::new(
                self.orbit_radius * yaw.cos(),
                self.height + vertical_motion,
                self.orbit_radius * yaw.sin(),
            );
        let motion_vector = Vec3::new(
            -self.orbit_radius * yaw.sin() * 0.08,
            vertical_motion * 0.24,
            self.orbit_radius * yaw.cos() * 0.08,
        );
        let aperture_radius = (self.orbit_radius / 240.0).clamp(0.018, 0.065);

        Camera::look_at(
            origin,
            self.focus_point,
            Vec3::new(0.0, 1.0, 0.0),
            self.vertical_fov,
            aspect_ratio,
        )
        .with_physical_lens(aperture_radius, 0.016, motion_vector)
    }

    pub fn distance_to_focus(&self) -> f64 {
        (self.orbit_radius * self.orbit_radius + self.height * self.height).sqrt()
    }
}

// =====================================================================
// Event bus
// =====================================================================

#[derive(Debug, Clone)]
pub enum EngineEvent {
    FrameStarted { frame_index: u64, target_ms: f64 },
    SimulationAdvanced { body_count: usize },
    ScenePrepared { node_count: usize },
    AudioMixed { master_gain: f64 },
    NetworkSynchronized { checksum: u64, clients: usize },
    FrameRendered { pixels: usize, output_path: String },
}

#[derive(Debug, Default, Clone)]
pub struct EventSummary {
    pub last_frame_index: u64,
    pub target_ms: f64,
    pub body_count: usize,
    pub node_count: usize,
    pub master_gain: f64,
    pub checksum: u64,
    pub clients: usize,
    pub pixels: usize,
    pub output_path: String,
}

#[derive(Debug, Default, Clone)]
pub struct EventBus {
    pending: Vec<EngineEvent>,
    history: Vec<EngineEvent>,
}

impl EventBus {
    pub fn push(&mut self, event: EngineEvent) {
        self.history.push(event.clone());
        self.pending.push(event);
    }

    pub fn drain(&mut self) -> Vec<EngineEvent> {
        std::mem::take(&mut self.pending)
    }

    pub fn history_len(&self) -> usize {
        self.history.len()
    }

    pub fn summarize_history(&self) -> EventSummary {
        let mut summary = EventSummary::default();

        for event in &self.history {
            match event {
                EngineEvent::FrameStarted {
                    frame_index,
                    target_ms,
                } => {
                    summary.last_frame_index = *frame_index;
                    summary.target_ms = *target_ms;
                }
                EngineEvent::SimulationAdvanced { body_count } => {
                    summary.body_count = *body_count;
                }
                EngineEvent::ScenePrepared { node_count } => {
                    summary.node_count = *node_count;
                }
                EngineEvent::AudioMixed { master_gain } => {
                    summary.master_gain = *master_gain;
                }
                EngineEvent::NetworkSynchronized { checksum, clients } => {
                    summary.checksum = *checksum;
                    summary.clients = *clients;
                }
                EngineEvent::FrameRendered {
                    pixels,
                    output_path,
                } => {
                    summary.pixels = *pixels;
                    summary.output_path = output_path.clone();
                }
            }
        }

        summary
    }
}

// =====================================================================
// Physics bookkeeping
// =====================================================================

use super::scene::celestial::CelestialBody;

#[derive(Debug, Clone)]
pub struct PhysicsManager {
    body_count: usize,
    total_mass: f64,
    total_kinetic_energy: f64,
    total_momentum: f64,
    average_orbital_radius: f64,
    net_gravity: f64,
    stability: f64,
}

impl PhysicsManager {
    pub fn from_bodies(bodies: &[CelestialBody]) -> Self {
        let total_mass: f64 = bodies.iter().map(|b| b.mass).sum();
        let center = if total_mass > f64::EPSILON {
            bodies
                .iter()
                .fold(Vec3::ZERO, |acc, b| acc + b.position * b.mass)
                / total_mass
        } else {
            Vec3::ZERO
        };

        let avg_orbital = if bodies.is_empty() {
            0.0
        } else {
            bodies
                .iter()
                .map(|b| (b.position - center).length())
                .sum::<f64>()
                / bodies.len() as f64
        };

        let gravity = bodies.iter().enumerate().fold(0.0_f64, |acc, (i, a)| {
            acc + bodies.iter().skip(i + 1).fold(0.0_f64, |g, b| {
                let dist = (a.position - b.position).length().max(0.01);
                g + a.mass * b.mass / (dist * dist)
            })
        });

        Self {
            body_count: bodies.len(),
            total_mass,
            total_kinetic_energy: total_mass * 0.5,
            total_momentum: total_mass * avg_orbital * 0.01,
            average_orbital_radius: avg_orbital,
            net_gravity: gravity,
            stability: 1.0 / (1.0 + gravity * 0.001),
        }
    }

    pub fn rebuild_from_bodies(&mut self, bodies: &[CelestialBody]) {
        *self = Self::from_bodies(bodies);
    }

    pub fn body_count(&self) -> usize {
        self.body_count
    }

    pub fn total_kinetic_energy(&self) -> f64 {
        self.total_kinetic_energy
    }

    pub fn total_momentum(&self) -> f64 {
        self.total_momentum
    }

    pub fn average_orbital_radius(&self) -> f64 {
        self.average_orbital_radius
    }

    pub fn net_gravity_measure(&self) -> f64 {
        self.net_gravity
    }

    pub fn stability_score(&self) -> f64 {
        self.stability
    }
}

// =====================================================================
// Resource manager
// =====================================================================

use crate::rendering::environment::procedural::ProceduralEnvironment;

#[derive(Debug, Clone, Copy)]
pub struct EnvironmentData {
    pub sky_top: Vec3,
    pub sky_bottom: Vec3,
    pub sun_direction: Vec3,
    pub sun_color: Vec3,
    pub sun_intensity: f64,
    pub sun_angular_radius: f64,
    pub exposure: f64,
    pub solar_elevation: f64,
}

impl Default for EnvironmentData {
    fn default() -> Self {
        Self {
            sky_top: Vec3::new(0.010, 0.016, 0.045),
            sky_bottom: Vec3::new(0.001, 0.001, 0.006),
            sun_direction: Vec3::new(-0.55, -0.85, -0.45).normalize(),
            sun_color: Vec3::new(1.0, 0.95, 0.88),
            sun_intensity: 1.65,
            sun_angular_radius: 0.045,
            exposure: 1.22,
            solar_elevation: 0.52,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ResourceManager {
    environment: EnvironmentData,
    surface_detail_scale: f64,
    hdri: ProceduralEnvironment,
}

impl ResourceManager {
    pub fn cinematic() -> Self {
        Self {
            environment: EnvironmentData::default(),
            surface_detail_scale: 1.0,
            hdri: ProceduralEnvironment::cinematic_space(),
        }
    }

    pub fn environment(&self) -> &EnvironmentData {
        &self.environment
    }

    pub fn surface_detail_scale(&self) -> f64 {
        self.surface_detail_scale
    }

    pub fn hdri(&self) -> &ProceduralEnvironment {
        &self.hdri
    }
}

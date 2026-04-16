//! Public crate API re-exports and compatibility helpers.

/// Modules d'intelligence artificielle et d'analyse de scène.
pub mod ai;
/// Contrôleurs de caméra de haut niveau.
pub mod camera;
/// Point d'entrée principal du moteur (`EngineApi`).
pub mod engine;
/// Catalogue de matériaux PBR prédéfinis.
pub mod materials;
/// Objets de scène préselectionnés (étoiles, planètes, bâtiments…).
pub mod objects;
/// Constructeurs et descripteurs de scène.
pub mod scenes;
/// Types partagés : `Quality`, `RenderRequest`, `CameraDesc`, etc.
pub mod types;

pub use self::engine::EngineApi;
/// API d'animation (clips, séquenceur de frames).
pub mod animation;

/// Public compatibility facade for the compute environment diagnostic.
pub fn diagnose_compute_environment() {
	let api = self::engine::EngineApi::new();
	api.diagnose_compute_environment(&self::engine::diagnostics::DiagnosticsOptions::default());
}


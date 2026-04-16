/// Contrôleur de caméra de haut niveau (orbit, look-at, cadrage auto).
pub mod controller;
/// Préréglages de caméra cinématiques prédéfinis.
pub mod presets;

pub use self::controller::CameraController;

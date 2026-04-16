/// Gestionnaire d'analyse IA (directives de qualité et d'exposition).
pub mod ai_manager;
/// Introspection des capacités du moteur exposées aux agents IA.
pub mod capabilities;
/// Conversion de prompts textuels en scènes.
pub mod prompt;
/// Façade de rendu orientée IA.
pub mod renderer;

pub use self::renderer::AiRenderer;

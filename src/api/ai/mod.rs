/// Heuristic AI manager used to derive runtime directives.
pub mod ai_manager;
/// Capability discovery exposed to AI clients.
pub mod capabilities;
/// Prompt-to-scene conversion helpers.
pub mod prompt;
/// AI rendering facade.
pub mod renderer;

/// Public AI renderer entry point.
pub use self::renderer::AiRenderer;

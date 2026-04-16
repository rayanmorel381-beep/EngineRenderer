//! `enginerenderer` — moteur de rendu 3D par lancer de rayons.
//!
//! Le point d'entrée principal est [`api::engine::EngineApi`]. La quasi-totalité
//! des types publics se trouve sous [`api`].
pub mod api;
pub(crate) mod core;

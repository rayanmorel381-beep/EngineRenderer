use crate::api::materials::catalog::MaterialCatalog;

/// The single entry point to the rendering engine.
#[derive(Debug)]
pub struct EngineApi {
    pub(crate) catalog: MaterialCatalog,
}

impl Default for EngineApi {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineApi {
    // -- construction -------------------------------------------------------

    /// Crée une nouvelle instance de l'API moteur.
    pub fn new() -> Self {
        Self {
            catalog: MaterialCatalog,
        }
    }

    // -- introspection ------------------------------------------------------

    /// Retourne une référence au catalogue de matériaux.
    pub fn materials(&self) -> &MaterialCatalog {
        &self.catalog
    }

    /// Retourne la liste statique de tous les noms de matériaux disponibles.
    pub fn material_names(&self) -> &'static [&'static str] {
        self.catalog.all_names()
    }
}

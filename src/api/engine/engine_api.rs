use crate::api::materials::catalog::MaterialCatalog;

#[derive(Debug)]
/// Main high-level API entry point for EngineRenderer.
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

    /// Creates a new engine API instance.
    pub fn new() -> Self {
        Self {
            catalog: MaterialCatalog,
        }
    }

    // -- introspection ------------------------------------------------------

    /// Returns the material catalog facade.
    pub fn materials(&self) -> &MaterialCatalog {
        &self.catalog
    }

    /// Returns all built-in material names.
    pub fn material_names(&self) -> &'static [&'static str] {
        self.catalog.all_names()
    }
}

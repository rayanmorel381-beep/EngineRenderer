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

    pub fn new() -> Self {
        Self {
            catalog: MaterialCatalog,
        }
    }

    // -- introspection ------------------------------------------------------

    pub fn materials(&self) -> &MaterialCatalog {
        &self.catalog
    }

    pub fn material_names(&self) -> &'static [&'static str] {
        self.catalog.all_names()
    }
}

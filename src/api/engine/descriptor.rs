use std::error::Error;

use crate::api::scene_descriptor::SceneDescriptor;
use crate::api::types::core::{RenderRequest, RenderResult};

use super::engine_api::EngineApi;

impl EngineApi {
    pub fn render_descriptor(
        &self,
        descriptor: SceneDescriptor,
        request: &RenderRequest,
    ) -> Result<RenderResult, Box<dyn Error>> {
        let builder = descriptor.into_builder();
        self.render(builder, request)
    }

    pub fn load_and_render<P: AsRef<std::path::Path>>(
        &self,
        scene_path: P,
        request: &RenderRequest,
    ) -> Result<RenderResult, Box<dyn Error>> {
        let descriptor = SceneDescriptor::load_from_file(scene_path)?;
        self.render_descriptor(descriptor, request)
    }
}

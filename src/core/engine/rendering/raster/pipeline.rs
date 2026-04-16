use crate::core::engine::math::{Mat4, Vec3};
use super::material::PbrMaterial;

pub struct VertexBuffer {
    pub bytes: Vec<u8>,
}

pub struct IndexBuffer {
    pub indices: Vec<u32>,
}

pub struct Mesh {
    pub vertex_buffer: VertexBuffer,
    pub index_buffer: IndexBuffer,
    pub material: PbrMaterial,
}

pub struct RasterPipeline {
    shader_id: u32,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    model_matrix: Mat4,
    light_pos: Vec3,
    view_pos: Vec3,
}

impl RasterPipeline {
    pub fn new() -> Self {
        RasterPipeline {
            shader_id: 0,
            view_matrix: Mat4::IDENTITY,
            projection_matrix: Mat4::IDENTITY,
            model_matrix: Mat4::IDENTITY,
            light_pos: Vec3::new(5.0, 5.0, 5.0),
            view_pos: Vec3::new(0.0, 0.0, 5.0),
        }
    }

    pub fn set_shader(&mut self, shader_id: u32) {
        self.shader_id = shader_id;
    }

    pub fn set_view_matrix(&mut self, matrix: Mat4) {
        self.view_matrix = matrix;
    }

    pub fn set_projection_matrix(&mut self, matrix: Mat4) {
        self.projection_matrix = matrix;
    }

    pub fn set_model_matrix(&mut self, matrix: Mat4) {
        self.model_matrix = matrix;
    }

    pub fn set_light_pos(&mut self, pos: Vec3) {
        self.light_pos = pos;
    }

    pub fn set_view_pos(&mut self, pos: Vec3) {
        self.view_pos = pos;
    }

    pub fn render(&self, _mesh: &Mesh) -> Result<(), String> {
        Ok(())
    }

    pub fn clear(&self, _color: [f32; 4]) {
    }

    pub fn set_viewport(&self, _x: i32, _y: i32, _width: i32, _height: i32) {
    }
}

/// Scene descriptor builders.
pub mod builder;
/// Ready-to-use scene presets.
pub mod presets;

pub use self::presets::*;
pub use self::builder::{
    AreaLightEntry, MAX_MATERIAL_NAME_LEN, MAX_SCENE_AREA_LIGHTS, MAX_SCENE_FILE_SIZE,
    MAX_SCENE_SPHERES, MAX_SCENE_TRIANGLES, SceneDescriptor, SphereEntry, TriangleEntry,
};

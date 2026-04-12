//! Core ray-tracing pipeline: BVH acceleration, camera model,
//! vector math, primitives, shading, and multi-threaded CPU tracer.

pub mod acceleration;
pub mod camera;
pub mod math;
pub mod primitives;
pub mod scene;
pub mod shading;
pub mod tracer;

pub use camera::Camera;
pub use math::Vec3;
pub use primitives::{Material, Ray, Sphere, Triangle};
pub use scene::{AreaLight, DirectionalLight, Scene};
pub use tracer::{BvhStats, CpuRayTracer, Image, RenderConfig};

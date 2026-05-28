//! `Renderer` struct definition, `Debug` impl, and shared internal helpers.

use std::sync::{Arc, Mutex, MutexGuard};

use crate::core::engine::acces_hardware::{
    CpuProfile, GpuRenderBackend, HardwareCapabilities, RamRuntimeConfig,
};
use crate::core::engine::rendering::{
    effects::{
        decals::decal_pass::Decal,
        particles::gpu_particles::GpuParticleSystem,
    },
    lod::manager::LodManager,
    materials::sss::{SssPass, SssProfile},
    mesh::skinning::{AnimationClip, Skeleton, SkinnedMesh},
    postprocessing::{
        fsr::FsrConfig,
        ssr::{IblProbe, SsrConfig, SsrPass},
        svgf::SvgfDenoiser,
        taa::TaaAccumulator,
    },
    sdf::WorldSdf,
    raytracing::{
        CpuRayTracer,
        acceleration::BvhNode,
        caustics::{CausticPass, PhotonMap},
        ddgi::{DdgiVolume, ProbeGrid},
        rtao::RtaoConfig,
    },
    shader_dispatcher::AdaptiveComputeDispatcher,
    terrain::{
        cdlod::{CdlodTerrain, HeightMap},
        foliage::{FoliageInstance, FoliageLayer},
    },
};

#[derive(Debug, Clone)]
pub(super) struct CachedBvh {
    pub signature: u64,
    pub object_count: usize,
    pub triangle_count: usize,
    pub bvh: Arc<BvhNode>,
}

pub struct Renderer {
    pub(super) width: usize,
    pub(super) height: usize,
    pub(super) tracer: CpuRayTracer,
    pub(super) lod_manager: LodManager,
    pub hw_caps: HardwareCapabilities,
    pub(super) cpu_profile: CpuProfile,
    pub(super) ram_config: RamRuntimeConfig,
    pub(super) gpu: Option<GpuRenderBackend>,
    pub(super) bvh_cache: Mutex<Option<CachedBvh>>,
    pub(super) compute_dispatcher: Mutex<AdaptiveComputeDispatcher>,
    pub(super) taa: Mutex<TaaAccumulator>,
    pub(super) svgf: Mutex<SvgfDenoiser>,
    pub(super) world_sdf: Mutex<Option<WorldSdf>>,
    pub(super) ddgi: Mutex<DdgiVolume>,
    pub(super) rtao_config: RtaoConfig,
    pub(super) photon_map: Mutex<PhotonMap>,
    pub(super) caustic_pass: CausticPass,
    pub(super) sss_pass: SssPass,
    pub(super) ssr_pass: SsrPass,
    pub(super) ibl_probe: IblProbe,
    pub(super) fsr_config: Option<FsrConfig>,
    pub(super) particles: Mutex<GpuParticleSystem>,
    pub(super) decals: Vec<Decal>,
    pub(super) skeletons: Mutex<Vec<Skeleton>>,
    pub(super) skinned_meshes: Mutex<Vec<SkinnedMesh>>,
    pub(super) animation_clips: Vec<AnimationClip>,
    pub(super) anim_state_machine: Mutex<crate::core::animation::state_machine::AnimStateMachine>,
    pub(super) texture_streamer: Mutex<crate::core::engine::rendering::texture::virtual_texture::TextureStreamer>,
    pub(super) terrain: Option<CdlodTerrain>,
    pub(super) foliage_layer: FoliageLayer,
    pub(super) foliage_instances: Mutex<Vec<FoliageInstance>>,
    pub(super) raster_pipeline: crate::core::engine::rendering::raster::pipeline::RasterPipeline,
    pub(super) secondary_motion: Mutex<crate::core::animation::secondary_motion::SecondaryMotionSystem>,
    pub(super) render_thread: Option<crate::core::engine::rendering::renderer::render_thread::RenderThread>,
    pub(super) job_system: crate::core::scheduler::job_system::JobSystem,
}

impl std::fmt::Debug for Renderer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Renderer")
            .field("width", &self.width)
            .field("height", &self.height)
            .field("lod_manager", &self.lod_manager)
            .field("hw_caps", &self.hw_caps)
            .field("cpu_profile", &self.cpu_profile)
            .field("ram_config", &self.ram_config)
            .field("gpu", &self.gpu)
            .finish()
    }
}

impl Renderer {
    pub(super) fn lock_unpoisoned<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
        match mutex.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        }
    }

    pub fn build_ddgi_from_scene_bounds(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> DdgiVolume {
        let origin = crate::core::engine::rendering::raytracing::Vec3::new(min_x, min_y, min_z);
        let extent = crate::core::engine::rendering::raytracing::Vec3::new(max_x - min_x, max_y - min_y, max_z - min_z);
        let grid = ProbeGrid::new(origin, extent * (1.0 / 5.0), [6, 4, 6]);
        DdgiVolume::new(grid)
    }

    pub fn set_terrain(&mut self, heightmap: HeightMap, max_lod: u32, base_range: f64) {
        self.terrain = Some(CdlodTerrain::new(heightmap, max_lod, base_range));
    }

    pub fn add_decal(&mut self, decal: Decal) {
        self.decals.push(decal);
    }

    pub fn add_animation_clip(&mut self, clip: AnimationClip) {
        self.animation_clips.push(clip);
    }

    pub fn add_skeleton(&mut self, skeleton: Skeleton) {
        Self::lock_unpoisoned(&self.skeletons).push(skeleton);
    }

    pub fn add_skinned_mesh(&mut self, mesh: SkinnedMesh) {
        Self::lock_unpoisoned(&self.skinned_meshes).push(mesh);
    }

    pub fn set_fsr(&mut self, out_width: usize, out_height: usize) {
        self.fsr_config = Some(FsrConfig::new(self.width, self.height, out_width, out_height));
    }

    pub fn set_sss_profile(&mut self, profile: SssProfile, kernel_radius: usize, samples: u32) {
        self.sss_pass = SssPass::new(profile, kernel_radius, samples);
    }

    pub fn set_ssr_config(&mut self, config: SsrConfig) {
        self.ssr_pass = SsrPass::new(config);
    }
}

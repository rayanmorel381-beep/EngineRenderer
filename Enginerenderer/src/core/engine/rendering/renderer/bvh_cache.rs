//! Persistent + in-memory BVH cache for `Scene` geometry signatures.

use std::sync::Arc;

use crate::core::engine::acces_hardware;
use crate::core::engine::rendering::raytracing::{Scene, acceleration::BvhNode};

use super::state::{CachedBvh, Renderer};

impl Renderer {
    fn persistent_bvh_cache_path(signature: u64, object_count: usize, triangle_count: usize) -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        path.push("enginerenderer-cache");
        path.push("bvh-cache");
        path.push(format!("{:016x}-{}-{}.bvh", signature, object_count, triangle_count));
        path
    }

    pub(super) fn cached_bvh_for_scene(&self, scene: &Scene) -> (Option<Arc<BvhNode>>, bool) {
        let signature = scene.geometry_signature();
        let object_count = scene.objects.len();
        let triangle_count = scene.triangles.len();
        let cache_path = Self::persistent_bvh_cache_path(signature, object_count, triangle_count);

        {
            let cache = Self::lock_unpoisoned(&self.bvh_cache);
            if let Some(cached) = cache.as_ref()
                && cached.signature == signature
                && cached.object_count == object_count
                && cached.triangle_count == triangle_count
            {
                return (Some(Arc::clone(&cached.bvh)), true);
            }
        }

        let t_load = acces_hardware::precise_timestamp_ns();
        if let Ok(loaded) = BvhNode::load_from_path(&cache_path, scene) {
            let loaded = Arc::new(loaded);
            let stats = loaded.stats();
            let load_ms = acces_hardware::elapsed_ms(t_load, acces_hardware::precise_timestamp_ns());
            crate::runtime_log!(
                "tracer: BVH disk-load {:.2}ms (nodes={} leaves={})",
                load_ms,
                stats.node_count,
                stats.leaf_count,
            );
            let mut cache = Self::lock_unpoisoned(&self.bvh_cache);
            *cache = Some(CachedBvh {
                signature,
                object_count,
                triangle_count,
                bvh: Arc::clone(&loaded),
            });
            return (Some(loaded), true);
        }

        let t_build = acces_hardware::precise_timestamp_ns();
        let built = BvhNode::build(scene).map(Arc::new);
        if let Some(ref bvh) = built {
            let stats = bvh.stats();
            let build_ms = acces_hardware::elapsed_ms(t_build, acces_hardware::precise_timestamp_ns());
            crate::runtime_log!(
                "tracer: BVH build {:.2}ms (nodes={} leaves={})",
                build_ms,
                stats.node_count,
                stats.leaf_count,
            );
            if let Err(error) = bvh.save_to_path(&cache_path) {
                crate::runtime_log!("tracer: BVH disk-save failed: {}", error);
            }
        }
        let mut cache = Self::lock_unpoisoned(&self.bvh_cache);
        *cache = built.as_ref().map(|bvh| CachedBvh {
            signature,
            object_count,
            triangle_count,
            bvh: Arc::clone(bvh),
        });
        (built, false)
    }
}

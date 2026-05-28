use std::path::PathBuf;

use crate::core::engine::rendering::mesh::asset::MeshAsset;
use super::glb_loader::GlbLoader;
use super::obj_loader::ObjLoader;

#[derive(Debug, Clone)]
pub struct ContentBundle {
    pub primary_meshes: Vec<MeshAsset>,
    pub cinematic_meshes: Vec<MeshAsset>,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct ContentLoader;

impl ContentLoader {
    pub fn load_showcase_bundle(&self) -> ContentBundle {
        let obj_loader = ObjLoader;
        let glb_loader = GlbLoader;
        let mut primary_meshes = Vec::new();
        let mut cinematic_meshes = Vec::new();

        for root in self.asset_roots() {
            primary_meshes.extend(obj_loader.load_directory(&root).unwrap_or_default());
            cinematic_meshes.extend(glb_loader.load_directory(&root).unwrap_or_default());
        }

        if primary_meshes.is_empty() {
            primary_meshes = vec![
                MeshAsset::procedural_asteroid("hero_asteroid", 1.2, 5),
                MeshAsset::procedural_asteroid("orbital_fragment", 0.6, 4),
            ];
        }

        if cinematic_meshes.is_empty() {
            cinematic_meshes = vec![
                MeshAsset::procedural_asteroid("background_massive", 2.1, 5),
                MeshAsset::procedural_asteroid("debris_cluster", 0.42, 4),
            ];
        }

        ContentBundle {
            primary_meshes,
            cinematic_meshes,
        }
    }

    fn asset_roots(&self) -> Vec<PathBuf> {
        let mut candidates = vec![
            PathBuf::from("assets"),
            PathBuf::from("models"),
            PathBuf::from("content"),
        ];

        if let Ok(current_dir) = std::env::current_dir() {
            candidates.push(current_dir.join("assets"));
            candidates.push(current_dir.join("models"));
            candidates.push(current_dir.join("content"));
        }

        let mut roots = Vec::new();
        for candidate in candidates {
            if candidate.is_dir() && !roots.iter().any(|existing| existing == &candidate) {
                roots.push(candidate);
            }
        }

        roots
    }
}

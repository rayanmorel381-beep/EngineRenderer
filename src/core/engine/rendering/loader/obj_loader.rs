use std::{fs, io, path::{Path, PathBuf}};

use crate::core::engine::rendering::{
    mesh::asset::MeshAsset, mesh::vertex::{MeshDescriptor, Vertex},
    raytracing::Vec3,
};

#[derive(Debug, Default, Clone, Copy)]
pub struct ObjLoader;

impl ObjLoader {
    pub fn load_embedded_showcase(&self) -> Vec<MeshAsset> {
        let mut meshes = self.load_directory("assets").unwrap_or_default();
        if meshes.is_empty() {
            meshes = vec![
                MeshAsset::procedural_asteroid("obj_embedded_hero", 1.35, 5),
                MeshAsset::procedural_asteroid("obj_embedded_detail", 0.48, 4),
            ];
        }
        meshes
    }

    pub fn load_directory<P: AsRef<Path>>(&self, directory: P) -> io::Result<Vec<MeshAsset>> {
        let mut files = Vec::new();
        self.collect_obj_files(directory.as_ref(), &mut files)?;

        let mut meshes = Vec::new();
        for path in files {
            if let Ok(mesh) = self.load_from_path(&path) {
                meshes.push(mesh);
            }
        }

        Ok(meshes)
    }

    pub fn load_from_path<P: AsRef<Path>>(&self, path: P) -> io::Result<MeshAsset> {
        let path = path.as_ref();
        let source = fs::read_to_string(path)?;
        let mut positions = Vec::<Vec3>::new();
        let mut texcoords = Vec::<(f64, f64)>::new();
        let mut normals = Vec::<Vec3>::new();
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<usize>::new();

        for raw_line in source.lines() {
            let line = raw_line.split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }

            let mut parts = line.split_whitespace();
            match parts.next() {
                Some("v") => {
                    let x = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
                    let y = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
                    let z = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
                    positions.push(Vec3::new(x, y, z));
                }
                Some("vt") => {
                    let u = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
                    let v = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
                    texcoords.push((u, v));
                }
                Some("vn") => {
                    let x = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
                    let y = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(0.0);
                    let z = parts.next().and_then(|value| value.parse::<f64>().ok()).unwrap_or(1.0);
                    normals.push(Vec3::new(x, y, z).normalize());
                }
                Some("f") => {
                    let face_tokens = parts.collect::<Vec<_>>();
                    if face_tokens.len() < 3 {
                        continue;
                    }

                    let mut face_vertices = face_tokens
                        .iter()
                        .filter_map(|token| self.parse_face_vertex(token, &positions, &texcoords, &normals))
                        .collect::<Vec<_>>();

                    if face_vertices.len() < 3 {
                        continue;
                    }

                    let face_normal = (face_vertices[1].position - face_vertices[0].position)
                        .cross(face_vertices[2].position - face_vertices[0].position)
                        .normalize();

                    for vertex in &mut face_vertices {
                        if vertex.normal.length_squared() <= f64::EPSILON {
                            vertex.normal = face_normal;
                        }
                    }

                    let base = vertices.len();
                    vertices.extend(face_vertices.iter().copied());
                    for triangle_index in 1..face_vertices.len() - 1 {
                        indices.push(base);
                        indices.push(base + triangle_index);
                        indices.push(base + triangle_index + 1);
                    }
                }
                _ => {}
            }
        }

        if vertices.is_empty() || indices.is_empty() {
            return Ok(MeshAsset::procedural_asteroid(
                path.file_stem().and_then(|value| value.to_str()).unwrap_or("obj_fallback"),
                1.0,
                16,
            ));
        }

        let bounding_radius = vertices
            .iter()
            .map(|vertex| vertex.position.length())
            .fold(0.0_f64, f64::max)
            .max(0.001);

        Ok(MeshAsset {
            name: path.file_stem().and_then(|value| value.to_str()).unwrap_or("obj_asset").to_string(),
            descriptor: MeshDescriptor {
                vertex_count: vertices.len(),
                triangle_count: indices.len() / 3,
                bounding_radius,
            },
            vertices,
            indices,
            preferred_material: None,
            base_translation: Vec3::ZERO,
            base_scale: Vec3::ONE,
            base_rotation: [0.0, 0.0, 0.0, 1.0],
        })
    }

    fn collect_obj_files(&self, directory: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
        if !directory.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_obj_files(&path, output)?;
            } else if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("obj"))
            {
                output.push(path);
            }
        }

        Ok(())
    }

    fn parse_face_vertex(
        &self,
        token: &str,
        positions: &[Vec3],
        texcoords: &[(f64, f64)],
        normals: &[Vec3],
    ) -> Option<Vertex> {
        let mut parts = token.split('/');
        let position_index = Self::resolve_index(Some(parts.next()?), positions.len())?;
        let texcoord_index = parts.next().and_then(|value| Self::resolve_index(Some(value), texcoords.len()));
        let normal_index = parts.next().and_then(|value| Self::resolve_index(Some(value), normals.len()));

        Some(Vertex {
            position: *positions.get(position_index)?,
            normal: normal_index.and_then(|index| normals.get(index).copied()).unwrap_or(Vec3::ZERO),
            uv: texcoord_index.and_then(|index| texcoords.get(index).copied()).map(|(u, v)| Vec3::new(u, v, 0.0)).unwrap_or(Vec3::ZERO),
            tangent: Vec3::ZERO,
        })
    }

    fn resolve_index(raw: Option<&str>, len: usize) -> Option<usize> {
        let raw = raw?.trim();
        if raw.is_empty() || len == 0 {
            return None;
        }

        let parsed = raw.parse::<isize>().ok()?;
        if parsed > 0 {
            Some((parsed as usize).saturating_sub(1))
        } else if parsed < 0 {
            let offset = len as isize + parsed;
            (offset >= 0).then_some(offset as usize)
        } else {
            None
        }
    }
}

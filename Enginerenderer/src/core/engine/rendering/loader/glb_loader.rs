//! Strict GLB (binary glTF 2.0) container loader.
//!
//! Validates the 12-byte header and walks chunks without panicking on
//! truncated or hostile input. Hard limits guard against pathological
//! sizes. Synthesises a [`MeshAsset`] from JSON node transforms and BIN
//! payload, falling back to deterministic procedural geometry when the
//! container does not expose decodable accessors.

use std::{fs, io, path::{Path, PathBuf}};

use crate::core::engine::rendering::{
    mesh::asset::MeshAsset, mesh::vertex::{MeshDescriptor, Vertex},
    raytracing::{Material, Vec3},
    texture::image_summary::TextureImageSummary,
};

/// Maximum on-disk size for a `.glb` file (256 MiB).
pub const MAX_GLB_FILE_SIZE: u64 = 256 * 1024 * 1024;
/// Maximum chunk payload size accepted while iterating the binary container.
pub const MAX_GLB_CHUNK_SIZE: usize = 200 * 1024 * 1024;
/// Minimum size of the 12-byte GLB header.
pub const GLB_HEADER_SIZE: usize = 12;
/// Expected magic bytes at offset 0.
pub const GLB_MAGIC: &[u8; 4] = b"glTF";
/// Supported GLB container version.
pub const GLB_SUPPORTED_VERSION: u32 = 2;
/// JSON chunk identifier (`'JSON'` little-endian).
pub const GLB_CHUNK_TYPE_JSON: u32 = 0x4E4F_534A;
/// BIN chunk identifier (`'BIN\0'` little-endian).
pub const GLB_CHUNK_TYPE_BIN: u32 = 0x004E_4942;

/// Errors produced while validating the binary GLB container.
#[derive(Debug)]
pub enum GlbLoadError {
    /// Underlying IO failure.
    Io(io::Error),
    /// File on disk exceeds [`MAX_GLB_FILE_SIZE`].
    FileTooLarge {
        /// Reported size on disk.
        size: u64,
        /// Configured limit.
        limit: u64,
    },
    /// File too small to contain the 12-byte header.
    HeaderTruncated {
        /// Actual byte count.
        size: usize,
    },
    /// First 4 bytes do not match `b"glTF"`.
    InvalidMagic {
        /// Bytes that were found.
        found: [u8; 4],
    },
    /// Container version is not [`GLB_SUPPORTED_VERSION`].
    UnsupportedVersion {
        /// Version number reported in the header.
        version: u32,
    },
    /// Declared total length disagrees with the actual file size.
    DeclaredLengthMismatch {
        /// Length advertised in the header.
        declared: u64,
        /// Length seen on disk.
        actual: usize,
    },
    /// A chunk header is truncated.
    ChunkHeaderTruncated {
        /// Byte offset of the chunk header.
        offset: usize,
    },
    /// A chunk payload is truncated according to the declared length.
    ChunkPayloadTruncated {
        /// Offset of the payload start.
        offset: usize,
        /// Length advertised by the chunk header.
        length: u64,
        /// Bytes that remain in the file.
        remaining: usize,
    },
    /// `offset + chunk_length` would overflow `usize`.
    ChunkLengthOverflow {
        /// Offset of the chunk header.
        offset: usize,
        /// Length advertised by the chunk header.
        length: u32,
    },
    /// A chunk advertises a payload larger than [`MAX_GLB_CHUNK_SIZE`].
    ChunkTooLarge {
        /// Length advertised by the chunk header.
        length: u64,
        /// Configured limit.
        limit: usize,
    },
}

impl std::fmt::Display for GlbLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::FileTooLarge { size, limit } => {
                write!(f, "glb file size {size} exceeds limit {limit}")
            }
            Self::HeaderTruncated { size } => {
                write!(f, "glb header truncated: {size} bytes (need {GLB_HEADER_SIZE})")
            }
            Self::InvalidMagic { found } => {
                write!(
                    f,
                    "invalid glb magic: 0x{:02x}{:02x}{:02x}{:02x}",
                    found[0], found[1], found[2], found[3]
                )
            }
            Self::UnsupportedVersion { version } => {
                write!(f, "unsupported glb version: {version} (need {GLB_SUPPORTED_VERSION})")
            }
            Self::DeclaredLengthMismatch { declared, actual } => {
                write!(f, "glb declared length {declared} != actual {actual}")
            }
            Self::ChunkHeaderTruncated { offset } => {
                write!(f, "glb chunk header truncated at offset {offset}")
            }
            Self::ChunkPayloadTruncated { offset, length, remaining } => {
                write!(
                    f,
                    "glb chunk payload truncated at offset {offset}: claims {length}, remaining {remaining}"
                )
            }
            Self::ChunkLengthOverflow { offset, length } => {
                write!(
                    f,
                    "glb chunk length overflow at offset {offset}: length {length}"
                )
            }
            Self::ChunkTooLarge { length, limit } => {
                write!(f, "glb chunk size {length} exceeds limit {limit}")
            }
        }
    }
}

impl std::error::Error for GlbLoadError {}

impl From<io::Error> for GlbLoadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<GlbLoadError> for io::Error {
    fn from(value: GlbLoadError) -> Self {
        match value {
            GlbLoadError::Io(err) => err,
            other => io::Error::new(io::ErrorKind::InvalidData, other.to_string()),
        }
    }
}

/// Validated GLB header.
#[derive(Debug, Clone, Copy)]
pub struct GlbHeader {
    /// Container version (always [`GLB_SUPPORTED_VERSION`] when validated).
    pub version: u32,
    /// Declared total length of the container, in bytes.
    pub declared_length: u32,
}

/// Validates the 12-byte GLB header and returns its parsed form.
///
/// Strict version: rejects truncated input, wrong magic, unsupported version,
/// and any mismatch between the declared total length and the actual buffer
/// size. Used by [`GlbLoader::load_from_path`] before any chunk iteration.
pub fn validate_glb_header(bytes: &[u8]) -> Result<GlbHeader, GlbLoadError> {
    if bytes.len() < GLB_HEADER_SIZE {
        return Err(GlbLoadError::HeaderTruncated { size: bytes.len() });
    }
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&bytes[0..4]);
    if &magic != GLB_MAGIC {
        return Err(GlbLoadError::InvalidMagic { found: magic });
    }
    let version = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
    if version != GLB_SUPPORTED_VERSION {
        return Err(GlbLoadError::UnsupportedVersion { version });
    }
    let declared_length = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]);
    if (declared_length as u64) != (bytes.len() as u64) {
        return Err(GlbLoadError::DeclaredLengthMismatch {
            declared: declared_length as u64,
            actual: bytes.len(),
        });
    }
    Ok(GlbHeader {
        version,
        declared_length,
    })
}

/// Iterates the chunks of a validated GLB container without panicking.
///
/// Each item is `(chunk_type, payload_slice)`. Returns an error variant on
/// truncated chunks, overflow on `offset + chunk_length`, or chunk sizes
/// above [`MAX_GLB_CHUNK_SIZE`].
pub fn iter_glb_chunks(bytes: &[u8]) -> Result<Vec<(u32, &[u8])>, GlbLoadError> {
    validate_glb_header(bytes)?;
    let mut chunks = Vec::new();
    let mut offset = GLB_HEADER_SIZE;
    while offset < bytes.len() {
        if offset + 8 > bytes.len() {
            return Err(GlbLoadError::ChunkHeaderTruncated { offset });
        }
        let chunk_length = u32::from_le_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        let chunk_type = u32::from_le_bytes([
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        let chunk_length_usize = usize::try_from(chunk_length).map_err(|_| {
            GlbLoadError::ChunkLengthOverflow {
                offset,
                length: chunk_length,
            }
        })?;
        if chunk_length_usize > MAX_GLB_CHUNK_SIZE {
            return Err(GlbLoadError::ChunkTooLarge {
                length: chunk_length as u64,
                limit: MAX_GLB_CHUNK_SIZE,
            });
        }
        let payload_start = offset
            .checked_add(8)
            .ok_or(GlbLoadError::ChunkLengthOverflow {
                offset,
                length: chunk_length,
            })?;
        let payload_end = payload_start.checked_add(chunk_length_usize).ok_or(
            GlbLoadError::ChunkLengthOverflow {
                offset,
                length: chunk_length,
            },
        )?;
        if payload_end > bytes.len() {
            return Err(GlbLoadError::ChunkPayloadTruncated {
                offset: payload_start,
                length: chunk_length as u64,
                remaining: bytes.len().saturating_sub(payload_start),
            });
        }
        chunks.push((chunk_type, &bytes[payload_start..payload_end]));
        offset = payload_end;
    }
    Ok(chunks)
}

#[derive(Debug, Clone)]
struct GltfNodeTransform {
    name: Option<String>,
    translation: Vec3,
    scale: Vec3,
    rotation: [f64; 4],
}

/// Stateless GLB (binary glTF 2.0) loader entry point.
#[derive(Debug, Default, Clone, Copy)]
pub struct GlbLoader;

impl GlbLoader {
    /// Loads GLB assets shipped with the engine, falling back to procedural
    /// asteroids if the `assets/` directory contains no `.glb` files.
    pub fn load_embedded_showcase(&self) -> Vec<MeshAsset> {
        let mut meshes = self.load_directory("assets").unwrap_or_default();
        if meshes.is_empty() {
            meshes = vec![
                MeshAsset::procedural_asteroid("glb_embedded_station", 1.8, 5),
                MeshAsset::procedural_asteroid("glb_embedded_shard", 0.52, 4),
            ];
        }
        meshes
    }

    /// Recursively loads every `.glb` file under `directory`. Files that
    /// individually fail to validate are skipped silently; IO errors while
    /// walking the directory are propagated.
    pub fn load_directory<P: AsRef<Path>>(&self, directory: P) -> io::Result<Vec<MeshAsset>> {
        let mut files = Vec::new();
        self.collect_glb_files(directory.as_ref(), &mut files)?;

        let mut meshes = Vec::new();
        for path in files {
            if let Ok(path_meshes) = self.load_from_path(&path) {
                meshes.extend(path_meshes);
            }
        }

        Ok(meshes)
    }

    /// Loads a GLB container from disk. Validates size against
    /// [`MAX_GLB_FILE_SIZE`], header against [`validate_glb_header`], and
    /// chunks against [`iter_glb_chunks`] before extracting nodes/textures.
    pub fn load_from_path<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<MeshAsset>> {
        let path = path.as_ref();
        let name = path.file_stem().and_then(|value| value.to_str()).unwrap_or("glb_asset");
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();

        if extension.eq_ignore_ascii_case("gltf") {
            let metadata = fs::metadata(path)?;
            if metadata.len() > MAX_GLB_FILE_SIZE {
                return Err(GlbLoadError::FileTooLarge {
                    size: metadata.len(),
                    limit: MAX_GLB_FILE_SIZE,
                }
                .into());
            }
            let json = fs::read_to_string(path)?;
            let material = self.material_from_json_and_images(path, &json);
            let node_instances = Self::extract_node_transforms(&json);
            let template = MeshAsset::procedural_asteroid(
                &format!("{}_gltf", name),
                1.1 + (node_instances.len() as f64 * 0.08).min(1.2),
                20 + node_instances.len().min(16) as u32,
            );
            return Ok(self.instantiate_from_nodes(name, template, material, node_instances));
        }

        let metadata = fs::metadata(path)?;
        if metadata.len() > MAX_GLB_FILE_SIZE {
            return Err(GlbLoadError::FileTooLarge {
                size: metadata.len(),
                limit: MAX_GLB_FILE_SIZE,
            }
            .into());
        }
        let bytes = fs::read(path)?;

        let header = validate_glb_header(&bytes).map_err(io::Error::from)?;
        let chunks = iter_glb_chunks(&bytes).map_err(io::Error::from)?;

        let json_chunk = chunks
            .iter()
            .find(|(chunk_type, _)| *chunk_type == GLB_CHUNK_TYPE_JSON)
            .map(|(_, payload)| {
                String::from_utf8_lossy(payload)
                    .trim_matches(char::from(0))
                    .to_string()
            });
        let bin_chunk = chunks
            .iter()
            .find(|(chunk_type, _)| *chunk_type == GLB_CHUNK_TYPE_BIN)
            .map(|(_, payload)| *payload);

        let pbr_material = json_chunk
            .as_deref()
            .and_then(|json| self.material_from_json_and_images(path, json));

        let payload_for_synthesis = bin_chunk.unwrap_or(&bytes[GLB_HEADER_SIZE..]);
        let max_synthesis_bytes = (header.declared_length as usize)
            .saturating_sub(GLB_HEADER_SIZE)
            .min(payload_for_synthesis.len());
        let payload_for_synthesis = &payload_for_synthesis[..max_synthesis_bytes];

        let mut points = Vec::new();
        for chunk in payload_for_synthesis.chunks_exact(12).take(8192) {
            let x = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f64;
            let y = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) as f64;
            let z = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]) as f64;

            if x.is_finite() && y.is_finite() && z.is_finite() && x.abs() < 1.0e6 && y.abs() < 1.0e6 && z.abs() < 1.0e6 {
                points.push(Vec3::new(x, y, z));
            }
        }

        let template = if points.len() < 3 {
            let radius = 1.0 + (header.version as f64 * 0.2);
            MeshAsset::procedural_asteroid(name, radius, 18 + header.version)
        } else {
            let centroid = points.iter().copied().fold(Vec3::ZERO, |acc, point| acc + point) / points.len() as f64;
            let max_radius = points
                .iter()
                .map(|point| (*point - centroid).length())
                .fold(0.0_f64, f64::max)
                .max(0.001);

            let vertices = points
                .iter()
                .enumerate()
                .map(|(index, point)| {
                    let position = (*point - centroid) / max_radius;
                    Vertex {
                        position,
                        normal: position.normalize(),
                        uv: Vec3::new(
                            (index as f64 / points.len() as f64).fract(),
                            ((position.y + 1.0) * 0.5).clamp(0.0, 1.0),
                            0.0,
                        ),
                        tangent: Vec3::ZERO,
                    }
                })
                .collect::<Vec<_>>();

            let mut indices = Vec::new();
            for base in (0..vertices.len().saturating_sub(2)).step_by(3) {
                indices.push(base);
                indices.push(base + 1);
                indices.push(base + 2);
            }

            if indices.is_empty() {
                MeshAsset::procedural_asteroid(name, 1.3, 18)
            } else {
                MeshAsset {
                    name: format!("{}_glb", name),
                    descriptor: MeshDescriptor {
                        vertex_count: vertices.len(),
                        triangle_count: indices.len() / 3,
                        bounding_radius: 1.0,
                    },
                    vertices,
                    indices,
                    preferred_material: pbr_material,
                    base_translation: Vec3::ZERO,
                    base_scale: Vec3::ONE,
                    base_rotation: [0.0, 0.0, 0.0, 1.0],
                }
            }
        };

        let node_instances = json_chunk
            .as_deref()
            .map(Self::extract_node_transforms)
            .unwrap_or_default();
        Ok(self.instantiate_from_nodes(name, template, pbr_material, node_instances))
    }

    fn collect_glb_files(&self, directory: &Path, output: &mut Vec<PathBuf>) -> io::Result<()> {
        if !directory.exists() {
            return Ok(());
        }

        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                self.collect_glb_files(&path, output)?;
            } else if path
                .extension()
                .and_then(|value| value.to_str())
                .is_some_and(|extension| extension.eq_ignore_ascii_case("glb") || extension.eq_ignore_ascii_case("gltf"))
            {
                output.push(path);
            }
        }

        Ok(())
    }

    fn material_from_json_and_images(&self, asset_path: &Path, json: &str) -> Option<Material> {
        let material = Self::extract_pbr_material(json)?;
        Some(self.apply_image_palette(asset_path.parent(), json, material))
    }

    fn apply_image_palette(&self, base_dir: Option<&Path>, json: &str, mut material: Material) -> Material {
        let image_uris = Self::extract_image_uris(json);
        if let Some((tint, detail)) = self.probe_texture_palette(base_dir, &image_uris) {
            material.albedo = material.albedo.lerp(tint, 0.34);
            material.sheen += tint * 0.04;
            material.texture_weight = (material.texture_weight + 0.12 + detail * 0.18).clamp(0.0, 1.0);
            material.normal_map_strength = (material.normal_map_strength + detail * 0.45).clamp(0.0, 3.0);
            material.uv_scale *= 1.0 + detail * 0.35;
        }
        material
    }

    fn probe_texture_palette(&self, base_dir: Option<&Path>, uris: &[String]) -> Option<(Vec3, f64)> {
        let base_dir = base_dir?;
        let mut accumulated = Vec3::ZERO;
        let mut total_weight = 0.0;
        let mut detail = 0.0;

        for uri in uris {
            let candidate = base_dir.join(uri);

            if let Some(summary) = TextureImageSummary::from_path(&candidate) {
                let texel_count = (summary.width as f64 * summary.height as f64).max(1.0);
                let weight = 1.0 + texel_count.ln().min(14.0) * 0.04;
                accumulated += summary.average_color * weight;
                total_weight += weight;
                detail += summary.detail;
                continue;
            }

            let Ok(bytes) = fs::read(candidate) else {
                continue;
            };
            if bytes.is_empty() {
                continue;
            }

            let mut r = 0.0;
            let mut g = 0.0;
            let mut b = 0.0;
            let mut count = 0.0;
            for chunk in bytes.chunks(3).take(4096) {
                r += *chunk.first().unwrap_or(&0) as f64 / 255.0;
                g += *chunk.get(1).unwrap_or(&0) as f64 / 255.0;
                b += *chunk.get(2).unwrap_or(&0) as f64 / 255.0;
                count += 1.0;
            }

            if count > 0.0 {
                let tint = Vec3::new(r / count, g / count, b / count);
                let weight = 1.0 + (bytes.len() as f64).ln().min(12.0) * 0.05;
                accumulated += tint * weight;
                total_weight += weight;
                detail += ((bytes.len() as f64).ln() / 12.0).clamp(0.0, 1.0);
            }
        }

        if total_weight <= f64::EPSILON {
            None
        } else {
            Some(((accumulated / total_weight).clamp(0.0, 1.0), (detail / uris.len().max(1) as f64).clamp(0.0, 1.0)))
        }
    }

    fn instantiate_from_nodes(
        &self,
        name: &str,
        template: MeshAsset,
        pbr_material: Option<Material>,
        node_instances: Vec<GltfNodeTransform>,
    ) -> Vec<MeshAsset> {
        let template = if let Some(material) = pbr_material {
            template.with_material(material)
        } else {
            template
        };

        if node_instances.is_empty() {
            return vec![template];
        }

        node_instances
            .into_iter()
            .enumerate()
            .map(|(index, node)| {
                let mut asset = template
                    .clone()
                    .with_transform(node.translation, node.scale, Some(node.rotation));
                asset.name = node
                    .name
                    .map(|node_name| format!("{}_{}", name, node_name))
                    .unwrap_or_else(|| format!("{}_node_{}", name, index));
                asset
            })
            .collect()
    }

    fn extract_node_transforms(json: &str) -> Vec<GltfNodeTransform> {
        let Some(nodes_block) = Self::extract_array_block(json, "\"nodes\"") else {
            return Vec::new();
        };

        Self::split_top_level_objects(nodes_block)
            .into_iter()
            .map(|block| {
                let translation_values = Self::extract_array_after(block, "\"translation\"", 3)
                    .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);
                let scale_values = Self::extract_array_after(block, "\"scale\"", 3)
                    .unwrap_or_else(|| vec![1.0, 1.0, 1.0]);
                let rotation_values = Self::extract_array_after(block, "\"rotation\"", 4)
                    .unwrap_or_else(|| vec![0.0, 0.0, 0.0, 1.0]);

                GltfNodeTransform {
                    name: Self::extract_string_after(block, "\"name\""),
                    translation: Vec3::new(
                        translation_values[0],
                        translation_values[1],
                        translation_values[2],
                    ),
                    scale: Vec3::new(scale_values[0], scale_values[1], scale_values[2]),
                    rotation: [
                        rotation_values[0],
                        rotation_values[1],
                        rotation_values[2],
                        rotation_values[3],
                    ],
                }
            })
            .collect()
    }

    fn extract_pbr_material(json: &str) -> Option<Material> {
        let base_color = Self::extract_array_after(json, "\"baseColorFactor\"", 4)
            .unwrap_or_else(|| vec![0.78, 0.80, 0.84, 1.0]);
        if base_color.len() < 3 {
            return None;
        }

        let emissive = Self::extract_array_after(json, "\"emissiveFactor\"", 3)
            .unwrap_or_else(|| vec![0.0, 0.0, 0.0]);
        let roughness = Self::extract_number_after(json, "\"roughnessFactor\"")
            .unwrap_or(0.38)
            .clamp(0.02, 0.98);
        let metallic = Self::extract_number_after(json, "\"metallicFactor\"")
            .unwrap_or(0.26)
            .clamp(0.0, 1.0);
        let transmission = Self::extract_number_after(json, "\"transmissionFactor\"")
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let ior = Self::extract_number_after(json, "\"ior\"")
            .unwrap_or(1.45)
            .max(1.0);
        let occlusion = Self::extract_number_after(json, "\"occlusionStrength\"")
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);
        let clearcoat = Self::extract_number_after(json, "\"clearcoatFactor\"")
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let iridescence = Self::extract_number_after(json, "\"iridescenceFactor\"")
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let emissive_strength = Self::extract_number_after(json, "\"emissiveStrength\"")
            .unwrap_or(1.0)
            .max(0.0);
        let specular_factor = Self::extract_number_after(json, "\"specularFactor\"")
            .unwrap_or(1.0)
            .clamp(0.0, 2.0);
        let thickness = Self::extract_number_after(json, "\"thicknessFactor\"")
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let attenuation_color = Self::extract_array_after(json, "\"attenuationColor\"", 3)
            .unwrap_or_else(|| vec![1.0, 1.0, 1.0]);
        let has_base_color_texture = json.contains("\"baseColorTexture\"");
        let has_normal_texture = json.contains("\"normalTexture\"");
        let has_occlusion_texture = json.contains("\"occlusionTexture\"");
        let has_emissive_texture = json.contains("\"emissiveTexture\"");
        let anisotropy = ((metallic * 0.55)
            + clearcoat * 0.25
            + (1.0 - roughness) * 0.20
            + if has_normal_texture { 0.12 } else { 0.0 })
            .clamp(0.0, 1.0);
        let subsurface = ((transmission * 0.65)
            + (1.0 - metallic) * 0.18
            + thickness * 0.45)
            .clamp(0.0, 1.0);
        let sheen = Vec3::new(base_color[0], base_color[1], base_color[2])
            * (if has_base_color_texture { 0.12 } else { 0.08 });
        let emissive_color = Vec3::new(emissive[0], emissive[1], emissive[2])
            * emissive_strength
            * if has_emissive_texture { 1.25 } else { 1.0 };
        let texture_weight = if has_base_color_texture { 0.78 } else { 0.48 };
        let normal_strength = if has_normal_texture { 1.45 } else { 0.85 };
        let uv_scale = if has_occlusion_texture || has_base_color_texture { 1.35 } else { 1.0 };
        let attenuated_albedo = Vec3::new(
            base_color[0] * attenuation_color[0],
            base_color[1] * attenuation_color[1],
            base_color[2] * attenuation_color[2],
        );

        Some(
            Material::new(
                attenuated_albedo,
                roughness,
                metallic,
                (0.18 + metallic * 0.62) * specular_factor,
                emissive_color,
            )
            .with_layers(
                if has_occlusion_texture { occlusion * 0.96 } else { occlusion },
                clearcoat.max(0.08 + (1.0 - roughness) * 0.20),
                sheen,
            )
            .with_transmission(transmission, ior)
            .with_optics(subsurface, anisotropy, iridescence)
            .with_texturing(texture_weight, normal_strength, uv_scale),
        )
    }

    fn extract_array_after(json: &str, key: &str, minimum_items: usize) -> Option<Vec<f64>> {
        let start = json.find(key)?;
        let after_key = &json[start + key.len()..];
        let bracket_start = after_key.find('[')? + 1;
        let remainder = &after_key[bracket_start..];
        let bracket_end = remainder.find(']')?;
        let values = remainder[..bracket_end]
            .split(',')
            .filter_map(|value| value.trim().parse::<f64>().ok())
            .collect::<Vec<_>>();

        (values.len() >= minimum_items).then_some(values)
    }

    fn extract_number_after(json: &str, key: &str) -> Option<f64> {
        let start = json.find(key)?;
        let after_key = &json[start + key.len()..];
        let colon = after_key.find(':')? + 1;
        let numeric = after_key[colon..]
            .chars()
            .skip_while(|character| character.is_whitespace())
            .take_while(|character| {
                character.is_ascii_digit()
                    || matches!(character, '.' | '-' | '+' | 'e' | 'E')
            })
            .collect::<String>();

        numeric.parse::<f64>().ok()
    }

    fn extract_image_uris(json: &str) -> Vec<String> {
        let Some(images_block) = Self::extract_array_block(json, "\"images\"") else {
            return Vec::new();
        };

        Self::split_top_level_objects(images_block)
            .into_iter()
            .filter_map(|block| Self::extract_string_after(block, "\"uri\""))
            .collect()
    }

    fn extract_string_after(json: &str, key: &str) -> Option<String> {
        let start = json.find(key)?;
        let after_key = &json[start + key.len()..];
        let colon = after_key.find(':')? + 1;
        let remainder = after_key[colon..].trim_start();
        let quoted = remainder.strip_prefix('"')?;
        let end = quoted.find('"')?;
        Some(quoted[..end].to_string())
    }

    fn extract_array_block<'a>(json: &'a str, key: &str) -> Option<&'a str> {
        let start = json.find(key)?;
        let after_key = &json[start + key.len()..];
        let bracket_start = after_key.find('[')?;
        let slice = &after_key[bracket_start..];
        let mut depth = 0isize;
        let mut in_string = false;
        let mut escape = false;

        for (index, character) in slice.char_indices() {
            if in_string {
                if escape {
                    escape = false;
                    continue;
                }
                match character {
                    '\\' => escape = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            match character {
                '"' => in_string = true,
                '[' => depth += 1,
                ']' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(&slice[1..index]);
                    }
                }
                _ => {}
            }
        }

        None
    }

    fn split_top_level_objects(block: &str) -> Vec<&str> {
        let mut objects = Vec::new();
        let mut depth = 0isize;
        let mut in_string = false;
        let mut escape = false;
        let mut start = None;

        for (index, character) in block.char_indices() {
            if in_string {
                if escape {
                    escape = false;
                    continue;
                }
                match character {
                    '\\' => escape = true,
                    '"' => in_string = false,
                    _ => {}
                }
                continue;
            }

            match character {
                '"' => in_string = true,
                '{' => {
                    if depth == 0 {
                        start = Some(index + 1);
                    }
                    depth += 1;
                }
                '}' => {
                    depth -= 1;
                    if depth == 0
                        && let Some(start_index) = start.take()
                    {
                        objects.push(&block[start_index..index]);
                    }
                }
                _ => {}
            }
        }

        objects
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_header(version: u32, total_length: u32) -> Vec<u8> {
        let mut buf = Vec::with_capacity(GLB_HEADER_SIZE);
        buf.extend_from_slice(GLB_MAGIC);
        buf.extend_from_slice(&version.to_le_bytes());
        buf.extend_from_slice(&total_length.to_le_bytes());
        buf
    }

    fn build_chunk(chunk_type: u32, payload: &[u8]) -> Vec<u8> {
        let mut buf = Vec::with_capacity(8 + payload.len());
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(&chunk_type.to_le_bytes());
        buf.extend_from_slice(payload);
        buf
    }

    fn build_glb(chunks: &[(u32, Vec<u8>)]) -> Vec<u8> {
        let mut total = GLB_HEADER_SIZE;
        let mut tail = Vec::new();
        for (ty, payload) in chunks {
            let chunk = build_chunk(*ty, payload);
            total += chunk.len();
            tail.extend(chunk);
        }
        let mut buf = build_header(GLB_SUPPORTED_VERSION, total as u32);
        buf.extend(tail);
        buf
    }

    #[test]
    fn validate_header_rejects_truncated_input() {
        let buf = vec![0u8; GLB_HEADER_SIZE - 1];
        match validate_glb_header(&buf) {
            Err(GlbLoadError::HeaderTruncated { size }) => assert_eq!(size, GLB_HEADER_SIZE - 1),
            other => panic!("expected HeaderTruncated, got {other:?}"),
        }
    }

    #[test]
    fn validate_header_rejects_invalid_magic() {
        let mut buf = build_header(GLB_SUPPORTED_VERSION, GLB_HEADER_SIZE as u32);
        buf[0] = b'X';
        match validate_glb_header(&buf) {
            Err(GlbLoadError::InvalidMagic { found }) => assert_eq!(found[0], b'X'),
            other => panic!("expected InvalidMagic, got {other:?}"),
        }
    }

    #[test]
    fn validate_header_rejects_unsupported_version() {
        let buf = build_header(1, GLB_HEADER_SIZE as u32);
        match validate_glb_header(&buf) {
            Err(GlbLoadError::UnsupportedVersion { version }) => assert_eq!(version, 1),
            other => panic!("expected UnsupportedVersion, got {other:?}"),
        }
    }

    #[test]
    fn validate_header_rejects_declared_length_mismatch() {
        let buf = build_header(GLB_SUPPORTED_VERSION, (GLB_HEADER_SIZE as u32) + 16);
        match validate_glb_header(&buf) {
            Err(GlbLoadError::DeclaredLengthMismatch { declared, actual }) => {
                assert_eq!(declared, (GLB_HEADER_SIZE as u64) + 16);
                assert_eq!(actual, GLB_HEADER_SIZE);
            }
            other => panic!("expected DeclaredLengthMismatch, got {other:?}"),
        }
    }

    #[test]
    fn validate_header_accepts_minimal_valid_buffer() {
        let buf = build_header(GLB_SUPPORTED_VERSION, GLB_HEADER_SIZE as u32);
        let header = validate_glb_header(&buf).expect("minimal header valid");
        assert_eq!(header.version, GLB_SUPPORTED_VERSION);
        assert_eq!(header.declared_length as usize, GLB_HEADER_SIZE);
    }

    #[test]
    fn iter_chunks_returns_empty_on_header_only_buffer() {
        let buf = build_header(GLB_SUPPORTED_VERSION, GLB_HEADER_SIZE as u32);
        let chunks = iter_glb_chunks(&buf).expect("header only is valid");
        assert!(chunks.is_empty());
    }

    #[test]
    fn iter_chunks_walks_json_and_bin() {
        let json = b"{\"a\":1}".to_vec();
        let bin = vec![1u8, 2, 3, 4];
        let buf = build_glb(&[
            (GLB_CHUNK_TYPE_JSON, json.clone()),
            (GLB_CHUNK_TYPE_BIN, bin.clone()),
        ]);
        let chunks = iter_glb_chunks(&buf).expect("two-chunk container valid");
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].0, GLB_CHUNK_TYPE_JSON);
        assert_eq!(chunks[0].1, json.as_slice());
        assert_eq!(chunks[1].0, GLB_CHUNK_TYPE_BIN);
        assert_eq!(chunks[1].1, bin.as_slice());
    }

    #[test]
    fn iter_chunks_rejects_truncated_chunk_header() {
        let mut buf = build_header(GLB_SUPPORTED_VERSION, (GLB_HEADER_SIZE as u32) + 4);
        buf.extend_from_slice(&[0u8, 0, 0, 0]);
        match iter_glb_chunks(&buf) {
            Err(GlbLoadError::ChunkHeaderTruncated { offset }) => {
                assert_eq!(offset, GLB_HEADER_SIZE);
            }
            other => panic!("expected ChunkHeaderTruncated, got {other:?}"),
        }
    }

    #[test]
    fn iter_chunks_rejects_payload_truncation() {
        let mut tail = Vec::new();
        tail.extend_from_slice(&16u32.to_le_bytes());
        tail.extend_from_slice(&GLB_CHUNK_TYPE_JSON.to_le_bytes());
        tail.extend_from_slice(&[1u8, 2, 3]);
        let total = GLB_HEADER_SIZE + tail.len();
        let mut buf = build_header(GLB_SUPPORTED_VERSION, total as u32);
        buf.extend(tail);
        match iter_glb_chunks(&buf) {
            Err(GlbLoadError::ChunkPayloadTruncated { length, remaining, .. }) => {
                assert_eq!(length, 16);
                assert_eq!(remaining, 3);
            }
            other => panic!("expected ChunkPayloadTruncated, got {other:?}"),
        }
    }

    #[test]
    fn iter_chunks_rejects_oversized_chunk() {
        let mut tail = Vec::new();
        let oversize = (MAX_GLB_CHUNK_SIZE as u32).saturating_add(1);
        tail.extend_from_slice(&oversize.to_le_bytes());
        tail.extend_from_slice(&GLB_CHUNK_TYPE_BIN.to_le_bytes());
        let total = GLB_HEADER_SIZE + tail.len();
        let mut buf = build_header(GLB_SUPPORTED_VERSION, total as u32);
        buf.extend(tail);
        match iter_glb_chunks(&buf) {
            Err(GlbLoadError::ChunkTooLarge { length, limit }) => {
                assert_eq!(length, oversize as u64);
                assert_eq!(limit, MAX_GLB_CHUNK_SIZE);
            }
            other => panic!("expected ChunkTooLarge, got {other:?}"),
        }
    }

    #[test]
    fn iter_chunks_rejects_chunk_extending_past_eof() {
        let mut tail = Vec::new();
        tail.extend_from_slice(&u32::MAX.to_le_bytes());
        tail.extend_from_slice(&GLB_CHUNK_TYPE_BIN.to_le_bytes());
        let total = GLB_HEADER_SIZE + tail.len();
        let mut buf = build_header(GLB_SUPPORTED_VERSION, total as u32);
        buf.extend(tail);
        match iter_glb_chunks(&buf) {
            Err(GlbLoadError::ChunkTooLarge { .. })
            | Err(GlbLoadError::ChunkPayloadTruncated { .. })
            | Err(GlbLoadError::ChunkLengthOverflow { .. }) => {}
            other => panic!("expected size/overflow rejection, got {other:?}"),
        }
    }

    #[test]
    fn glb_load_error_to_io_error_preserves_message() {
        let err = GlbLoadError::HeaderTruncated { size: 4 };
        let msg = err.to_string();
        let io_err: io::Error = err.into();
        assert_eq!(io_err.kind(), io::ErrorKind::InvalidData);
        assert!(io_err.to_string().contains(&msg));
    }

    #[test]
    fn deterministic_fuzz_validate_header_never_panics() {
        let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
        for _ in 0..512 {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            let len = (state as usize) % 64;
            let mut buf = vec![0u8; len];
            for (i, byte) in buf.iter_mut().enumerate() {
                *byte = ((state >> ((i % 8) * 8)) & 0xFF) as u8;
            }
            let _ = validate_glb_header(&buf);
        }
    }

    #[test]
    fn deterministic_fuzz_iter_chunks_never_panics() {
        let mut state: u64 = 0xD1B5_4A32_D192_ED03;
        for _ in 0..512 {
            state ^= state << 11;
            state ^= state >> 9;
            state ^= state << 23;
            let payload_len = (state as usize) % 96;
            let total = GLB_HEADER_SIZE + payload_len;
            let mut buf = build_header(GLB_SUPPORTED_VERSION, total as u32);
            for i in 0..payload_len {
                let mix = state.wrapping_mul(i as u64 + 1);
                buf.push((mix & 0xFF) as u8);
            }
            let _ = iter_glb_chunks(&buf);
        }
    }
}

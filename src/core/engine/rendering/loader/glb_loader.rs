use std::{fs, io, path::{Path, PathBuf}};

use crate::core::engine::rendering::{
    mesh::asset::MeshAsset, mesh::vertex::{MeshDescriptor, Vertex},
    raytracing::{Material, Vec3},
    texture::image_summary::TextureImageSummary,
};

#[derive(Debug, Clone)]
struct GltfNodeTransform {
    name: Option<String>,
    translation: Vec3,
    scale: Vec3,
    rotation: [f64; 4],
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GlbLoader;

impl GlbLoader {
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

    pub fn load_from_path<P: AsRef<Path>>(&self, path: P) -> io::Result<Vec<MeshAsset>> {
        let path = path.as_ref();
        let name = path.file_stem().and_then(|value| value.to_str()).unwrap_or("glb_asset");
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();

        if extension.eq_ignore_ascii_case("gltf") {
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

        let bytes = fs::read(path)?;
        let json_chunk = self.extract_json_chunk(&bytes);
        let pbr_material = json_chunk
            .as_deref()
            .and_then(|json| self.material_from_json_and_images(path, json));

        if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
            return Ok(vec![MeshAsset::procedural_asteroid(name, 1.4, 20)]);
        }

        let version = u32::from_le_bytes(bytes[4..8].try_into().unwrap_or([2, 0, 0, 0]));
        let declared_length = u32::from_le_bytes(bytes[8..12].try_into().unwrap_or([0, 0, 0, 0])) as usize;
        let capped_length = declared_length.min(bytes.len()).max(12);
        let payload = &bytes[12..capped_length];

        let mut points = Vec::new();
        for chunk in payload.chunks_exact(12).take(8192) {
            let x = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]) as f64;
            let y = f32::from_le_bytes([chunk[4], chunk[5], chunk[6], chunk[7]]) as f64;
            let z = f32::from_le_bytes([chunk[8], chunk[9], chunk[10], chunk[11]]) as f64;

            if x.is_finite() && y.is_finite() && z.is_finite() && x.abs() < 1.0e6 && y.abs() < 1.0e6 && z.abs() < 1.0e6 {
                points.push(Vec3::new(x, y, z));
            }
        }

        let template = if points.len() < 3 {
            let radius = 1.0 + (version as f64 * 0.2);
            MeshAsset::procedural_asteroid(name, radius, 18 + version)
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

    fn extract_json_chunk(&self, bytes: &[u8]) -> Option<String> {
        if bytes.len() < 20 || &bytes[0..4] != b"glTF" {
            return None;
        }

        let mut offset = 12usize;
        while offset + 8 <= bytes.len() {
            let chunk_length = u32::from_le_bytes(bytes[offset..offset + 4].try_into().ok()?) as usize;
            let chunk_type = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?);
            offset += 8;
            if offset + chunk_length > bytes.len() {
                break;
            }

            if chunk_type == 0x4E4F_534A {
                let json_bytes = &bytes[offset..offset + chunk_length];
                return Some(String::from_utf8_lossy(json_bytes).trim_matches(char::from(0)).to_string());
            }

            offset += chunk_length;
        }

        None
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

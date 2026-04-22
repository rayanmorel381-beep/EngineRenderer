//! Strict OBJ wavefront mesh loader.
//!
//! Parses a subset of the OBJ format (`v`, `vt`, `vn`, `f`) into a
//! [`MeshAsset`]. Rejects malformed input early with a precise
//! [`ObjLoadError`] carrying the line number, instead of silently producing
//! garbage geometry. Hard limits guard against pathological inputs.

use std::{fs, io, path::{Path, PathBuf}};

use crate::core::engine::rendering::{
    mesh::asset::MeshAsset, mesh::vertex::{MeshDescriptor, Vertex},
    raytracing::Vec3,
};

/// Maximum on-disk size accepted for an OBJ file (512 MiB).
pub const MAX_OBJ_FILE_SIZE: u64 = 512 * 1024 * 1024;
/// Maximum number of vertices produced by parsing.
pub const MAX_OBJ_VERTICES: usize = 8_000_000;
/// Maximum number of triangle indices produced by parsing.
pub const MAX_OBJ_INDICES: usize = 24_000_000;
/// Maximum number of vertices per polygonal face before triangulation.
pub const MAX_OBJ_FACE_VERTICES: usize = 1024;

/// Errors reported by the OBJ loader. All non-IO variants carry the
/// 1-based line number where the issue was detected.
#[derive(Debug)]
pub enum ObjLoadError {
    /// Underlying IO failure while reading the file.
    Io(io::Error),
    /// File on disk exceeds [`MAX_OBJ_FILE_SIZE`].
    FileTooLarge {
        /// Reported size on disk.
        size: u64,
        /// Configured limit.
        limit: u64,
    },
    /// File is zero bytes long.
    Empty,
    /// A token expected to be a float failed to parse.
    InvalidFloat {
        /// 1-based line number.
        line: usize,
        /// Offending token.
        token: String,
    },
    /// A float parsed but is `NaN` or infinite.
    NonFiniteFloat {
        /// 1-based line number.
        line: usize,
        /// Offending token.
        token: String,
    },
    /// A directive is missing one of its required components.
    MissingComponent {
        /// 1-based line number.
        line: usize,
        /// OBJ directive (`v`, `vt`, `vn`, `f`, ...).
        directive: &'static str,
    },
    /// A face index token failed to parse as an integer.
    InvalidIndex {
        /// 1-based line number.
        line: usize,
        /// Offending token.
        token: String,
    },
    /// A face index resolves outside the declared vertex/texcoord/normal array.
    IndexOutOfRange {
        /// 1-based line number.
        line: usize,
        /// Resolved 1-based index (negative = relative).
        index: isize,
        /// Length of the array being indexed.
        len: usize,
    },
    /// A face references index `0` (forbidden by the OBJ specification).
    ZeroIndex {
        /// 1-based line number.
        line: usize,
    },
    /// A face has fewer than 3 vertices.
    FaceTooSmall {
        /// 1-based line number.
        line: usize,
        /// Number of vertices found.
        vertices: usize,
    },
    /// A face has more vertices than [`MAX_OBJ_FACE_VERTICES`].
    FaceTooLarge {
        /// 1-based line number.
        line: usize,
        /// Number of vertices found.
        vertices: usize,
        /// Configured limit.
        limit: usize,
    },
    /// Vertex count would exceed [`MAX_OBJ_VERTICES`].
    TooManyVertices {
        /// Vertex count produced.
        count: usize,
        /// Configured limit.
        limit: usize,
    },
    /// Index count would exceed [`MAX_OBJ_INDICES`].
    TooManyIndices {
        /// Index count produced.
        count: usize,
        /// Configured limit.
        limit: usize,
    },
    /// Parsing succeeded but produced zero triangles.
    NoGeometry,
}

impl std::fmt::Display for ObjLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "io error: {err}"),
            Self::FileTooLarge { size, limit } => {
                write!(f, "obj file size {size} exceeds limit {limit}")
            }
            Self::Empty => write!(f, "obj file is empty"),
            Self::InvalidFloat { line, token } => {
                write!(f, "line {line}: invalid float '{token}'")
            }
            Self::NonFiniteFloat { line, token } => {
                write!(f, "line {line}: non-finite float '{token}'")
            }
            Self::MissingComponent { line, directive } => {
                write!(f, "line {line}: missing component for '{directive}'")
            }
            Self::InvalidIndex { line, token } => {
                write!(f, "line {line}: invalid index '{token}'")
            }
            Self::IndexOutOfRange { line, index, len } => {
                write!(f, "line {line}: index {index} out of range (len {len})")
            }
            Self::ZeroIndex { line } => {
                write!(f, "line {line}: zero index is forbidden by obj spec")
            }
            Self::FaceTooSmall { line, vertices } => {
                write!(f, "line {line}: face needs at least 3 vertices, found {vertices}")
            }
            Self::FaceTooLarge { line, vertices, limit } => {
                write!(f, "line {line}: face has {vertices} vertices, limit is {limit}")
            }
            Self::TooManyVertices { count, limit } => {
                write!(f, "vertex count {count} exceeds limit {limit}")
            }
            Self::TooManyIndices { count, limit } => {
                write!(f, "index count {count} exceeds limit {limit}")
            }
            Self::NoGeometry => write!(f, "no triangles produced from obj file"),
        }
    }
}

impl std::error::Error for ObjLoadError {}

impl From<io::Error> for ObjLoadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<ObjLoadError> for io::Error {
    fn from(value: ObjLoadError) -> Self {
        match value {
            ObjLoadError::Io(err) => err,
            other => io::Error::new(io::ErrorKind::InvalidData, other.to_string()),
        }
    }
}

/// Stateless OBJ loader entry point.
#[derive(Debug, Default, Clone, Copy)]
pub struct ObjLoader;

impl ObjLoader {
    /// Loads OBJ assets shipped with the engine, falling back to procedural
    /// asteroids if the `assets/` directory is empty or absent.
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

    /// Recursively loads every `.obj` file under `directory`. Files that
    /// individually fail to parse are skipped silently; IO errors while
    /// walking the directory are propagated.
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

    /// Loads a single OBJ file from disk. Validates size against
    /// [`MAX_OBJ_FILE_SIZE`] before reading, then delegates to
    /// [`Self::parse_str`].
    pub fn load_from_path<P: AsRef<Path>>(&self, path: P) -> io::Result<MeshAsset> {
        let path = path.as_ref();
        let metadata = fs::metadata(path)?;
        let size = metadata.len();
        if size > MAX_OBJ_FILE_SIZE {
            return Err(ObjLoadError::FileTooLarge { size, limit: MAX_OBJ_FILE_SIZE }.into());
        }
        if size == 0 {
            return Err(ObjLoadError::Empty.into());
        }

        let source = fs::read_to_string(path)?;
        let asset_name = path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("obj_asset")
            .to_string();

        Self::parse_str(&source, asset_name).map_err(io::Error::from)
    }

    /// Parses an OBJ source string into a [`MeshAsset`]. UTF-8 BOM is
    /// tolerated on the first line. Polygons with more than 3 vertices are
    /// triangulated by ear-clipping.
    pub fn parse_str(source: &str, asset_name: String) -> Result<MeshAsset, ObjLoadError> {
        let mut positions = Vec::<Vec3>::new();
        let mut texcoords = Vec::<(f64, f64)>::new();
        let mut normals = Vec::<Vec3>::new();
        let mut vertices = Vec::<Vertex>::new();
        let mut indices = Vec::<usize>::new();

        let mut iter = source.lines().enumerate();
        if let Some((_, first)) = iter.next() {
            let stripped = first.strip_prefix('\u{feff}').unwrap_or(first);
            Self::process_line(
                1,
                stripped,
                &mut positions,
                &mut texcoords,
                &mut normals,
                &mut vertices,
                &mut indices,
            )?;
        }

        for (zero_based, raw_line) in iter {
            Self::process_line(
                zero_based + 1,
                raw_line,
                &mut positions,
                &mut texcoords,
                &mut normals,
                &mut vertices,
                &mut indices,
            )?;
        }

        if vertices.is_empty() || indices.is_empty() {
            return Err(ObjLoadError::NoGeometry);
        }
        if vertices.len() > MAX_OBJ_VERTICES {
            return Err(ObjLoadError::TooManyVertices {
                count: vertices.len(),
                limit: MAX_OBJ_VERTICES,
            });
        }
        if indices.len() > MAX_OBJ_INDICES {
            return Err(ObjLoadError::TooManyIndices {
                count: indices.len(),
                limit: MAX_OBJ_INDICES,
            });
        }

        let bounding_radius = vertices
            .iter()
            .map(|vertex| vertex.position.length())
            .fold(0.0_f64, f64::max)
            .max(0.001);

        Ok(MeshAsset {
            name: asset_name,
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

    fn process_line(
        line_number: usize,
        raw_line: &str,
        positions: &mut Vec<Vec3>,
        texcoords: &mut Vec<(f64, f64)>,
        normals: &mut Vec<Vec3>,
        vertices: &mut Vec<Vertex>,
        indices: &mut Vec<usize>,
    ) -> Result<(), ObjLoadError> {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            return Ok(());
        }

        let mut parts = line.split_whitespace();
        match parts.next() {
            Some("v") => {
                let x = Self::parse_finite(parts.next(), line_number, "v")?;
                let y = Self::parse_finite(parts.next(), line_number, "v")?;
                let z = Self::parse_finite(parts.next(), line_number, "v")?;
                if positions.len() >= MAX_OBJ_VERTICES {
                    return Err(ObjLoadError::TooManyVertices {
                        count: positions.len() + 1,
                        limit: MAX_OBJ_VERTICES,
                    });
                }
                positions.push(Vec3::new(x, y, z));
            }
            Some("vt") => {
                let u = Self::parse_finite(parts.next(), line_number, "vt")?;
                let v = Self::parse_finite(parts.next(), line_number, "vt")?;
                texcoords.push((u, v));
            }
            Some("vn") => {
                let x = Self::parse_finite(parts.next(), line_number, "vn")?;
                let y = Self::parse_finite(parts.next(), line_number, "vn")?;
                let z = Self::parse_finite(parts.next(), line_number, "vn")?;
                let candidate = Vec3::new(x, y, z);
                let normalized = if candidate.length_squared() > 0.0 {
                    candidate.normalize()
                } else {
                    Vec3::new(0.0, 0.0, 1.0)
                };
                normals.push(normalized);
            }
            Some("f") => {
                let face_tokens = parts.collect::<Vec<_>>();
                if face_tokens.len() < 3 {
                    return Err(ObjLoadError::FaceTooSmall {
                        line: line_number,
                        vertices: face_tokens.len(),
                    });
                }
                if face_tokens.len() > MAX_OBJ_FACE_VERTICES {
                    return Err(ObjLoadError::FaceTooLarge {
                        line: line_number,
                        vertices: face_tokens.len(),
                        limit: MAX_OBJ_FACE_VERTICES,
                    });
                }

                let mut face_vertices = Vec::with_capacity(face_tokens.len());
                for token in &face_tokens {
                    face_vertices.push(Self::parse_face_vertex(
                        token,
                        line_number,
                        positions,
                        texcoords,
                        normals,
                    )?);
                }

                let face_normal = Self::compute_face_normal(&face_vertices);
                for vertex in &mut face_vertices {
                    if vertex.normal.length_squared() <= f64::EPSILON {
                        vertex.normal = face_normal;
                    }
                }

                let triangulation = Self::triangulate(&face_vertices, face_normal);
                if triangulation.is_empty() {
                    return Ok(());
                }

                if indices.len().saturating_add(triangulation.len() * 3) > MAX_OBJ_INDICES {
                    return Err(ObjLoadError::TooManyIndices {
                        count: indices.len() + triangulation.len() * 3,
                        limit: MAX_OBJ_INDICES,
                    });
                }
                if vertices.len().saturating_add(face_vertices.len()) > MAX_OBJ_VERTICES {
                    return Err(ObjLoadError::TooManyVertices {
                        count: vertices.len() + face_vertices.len(),
                        limit: MAX_OBJ_VERTICES,
                    });
                }

                let base = vertices.len();
                vertices.extend(face_vertices.iter().copied());
                for [a, b, c] in triangulation {
                    indices.push(base + a);
                    indices.push(base + b);
                    indices.push(base + c);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn parse_finite(
        token: Option<&str>,
        line_number: usize,
        directive: &'static str,
    ) -> Result<f64, ObjLoadError> {
        let raw = token.ok_or(ObjLoadError::MissingComponent {
            line: line_number,
            directive,
        })?;
        let value = raw.parse::<f64>().map_err(|_| ObjLoadError::InvalidFloat {
            line: line_number,
            token: raw.to_string(),
        })?;
        if !value.is_finite() {
            return Err(ObjLoadError::NonFiniteFloat {
                line: line_number,
                token: raw.to_string(),
            });
        }
        Ok(value)
    }

    fn parse_face_vertex(
        token: &str,
        line_number: usize,
        positions: &[Vec3],
        texcoords: &[(f64, f64)],
        normals: &[Vec3],
    ) -> Result<Vertex, ObjLoadError> {
        let mut parts = token.split('/');
        let position_raw = parts.next().ok_or(ObjLoadError::InvalidIndex {
            line: line_number,
            token: token.to_string(),
        })?;
        let position_index = Self::resolve_index(position_raw, positions.len(), line_number)?
            .ok_or(ObjLoadError::InvalidIndex {
                line: line_number,
                token: token.to_string(),
            })?;

        let texcoord_index = match parts.next() {
            Some(raw) if !raw.trim().is_empty() => {
                Self::resolve_index(raw, texcoords.len(), line_number)?
            }
            _ => None,
        };
        let normal_index = match parts.next() {
            Some(raw) if !raw.trim().is_empty() => {
                Self::resolve_index(raw, normals.len(), line_number)?
            }
            _ => None,
        };

        let position = *positions.get(position_index).ok_or(ObjLoadError::IndexOutOfRange {
            line: line_number,
            index: (position_index as isize) + 1,
            len: positions.len(),
        })?;

        let normal = match normal_index {
            Some(index) => *normals.get(index).ok_or(ObjLoadError::IndexOutOfRange {
                line: line_number,
                index: (index as isize) + 1,
                len: normals.len(),
            })?,
            None => Vec3::ZERO,
        };

        let uv = match texcoord_index {
            Some(index) => {
                let (u, v) = *texcoords.get(index).ok_or(ObjLoadError::IndexOutOfRange {
                    line: line_number,
                    index: (index as isize) + 1,
                    len: texcoords.len(),
                })?;
                Vec3::new(u, v, 0.0)
            }
            None => Vec3::ZERO,
        };

        Ok(Vertex {
            position,
            normal,
            uv,
            tangent: Vec3::ZERO,
        })
    }

    fn resolve_index(
        raw: &str,
        len: usize,
        line_number: usize,
    ) -> Result<Option<usize>, ObjLoadError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Ok(None);
        }
        let parsed = trimmed.parse::<i64>().map_err(|_| ObjLoadError::InvalidIndex {
            line: line_number,
            token: trimmed.to_string(),
        })?;
        if parsed == 0 {
            return Err(ObjLoadError::ZeroIndex { line: line_number });
        }
        if parsed > 0 {
            let idx = (parsed as u64).saturating_sub(1);
            if idx >= len as u64 {
                return Err(ObjLoadError::IndexOutOfRange {
                    line: line_number,
                    index: parsed as isize,
                    len,
                });
            }
            Ok(Some(idx as usize))
        } else {
            let offset = (len as i64).checked_add(parsed).ok_or(ObjLoadError::InvalidIndex {
                line: line_number,
                token: trimmed.to_string(),
            })?;
            if offset < 0 {
                return Err(ObjLoadError::IndexOutOfRange {
                    line: line_number,
                    index: parsed as isize,
                    len,
                });
            }
            Ok(Some(offset as usize))
        }
    }

    fn compute_face_normal(face: &[Vertex]) -> Vec3 {
        let mut accum = Vec3::ZERO;
        let count = face.len();
        for i in 0..count {
            let current = face[i].position;
            let next = face[(i + 1) % count].position;
            accum.x += (current.y - next.y) * (current.z + next.z);
            accum.y += (current.z - next.z) * (current.x + next.x);
            accum.z += (current.x - next.x) * (current.y + next.y);
        }
        if accum.length_squared() > 0.0 {
            accum.normalize()
        } else {
            Vec3::new(0.0, 0.0, 1.0)
        }
    }

    fn triangulate(face: &[Vertex], normal: Vec3) -> Vec<[usize; 3]> {
        let count = face.len();
        if count < 3 {
            return Vec::new();
        }
        if count == 3 {
            if Self::triangle_area_2x(face[0].position, face[1].position, face[2].position) <= f64::EPSILON {
                return Vec::new();
            }
            return vec![[0, 1, 2]];
        }

        let projected: Vec<(f64, f64)> = face
            .iter()
            .map(|vertex| Self::project_to_plane(vertex.position, normal))
            .collect();

        let signed_area: f64 = (0..count)
            .map(|i| {
                let (x0, y0) = projected[i];
                let (x1, y1) = projected[(i + 1) % count];
                x0 * y1 - x1 * y0
            })
            .sum::<f64>()
            * 0.5;

        let counter_clockwise = signed_area >= 0.0;
        let mut remaining: Vec<usize> = if counter_clockwise {
            (0..count).collect()
        } else {
            (0..count).rev().collect()
        };

        let mut output = Vec::with_capacity(count - 2);
        let mut guard = remaining.len() * remaining.len();

        while remaining.len() > 3 && guard > 0 {
            guard -= 1;
            let mut ear_found = false;
            let n = remaining.len();
            for i in 0..n {
                let prev = remaining[(i + n - 1) % n];
                let curr = remaining[i];
                let next = remaining[(i + 1) % n];
                let a = projected[prev];
                let b = projected[curr];
                let c = projected[next];
                if Self::signed_area_2d(a, b, c) <= 0.0 {
                    continue;
                }
                let mut contains_other = false;
                for &candidate in &remaining {
                    if candidate == prev || candidate == curr || candidate == next {
                        continue;
                    }
                    if Self::point_in_triangle(projected[candidate], a, b, c) {
                        contains_other = true;
                        break;
                    }
                }
                if contains_other {
                    continue;
                }
                output.push([prev, curr, next]);
                remaining.remove(i);
                ear_found = true;
                break;
            }
            if !ear_found {
                break;
            }
        }

        if remaining.len() == 3 {
            output.push([remaining[0], remaining[1], remaining[2]]);
        }

        output.retain(|tri| {
            Self::triangle_area_2x(face[tri[0]].position, face[tri[1]].position, face[tri[2]].position)
                > f64::EPSILON
        });

        output
    }

    fn project_to_plane(point: Vec3, normal: Vec3) -> (f64, f64) {
        let abs_x = normal.x.abs();
        let abs_y = normal.y.abs();
        let abs_z = normal.z.abs();
        if abs_x >= abs_y && abs_x >= abs_z {
            (point.y, point.z)
        } else if abs_y >= abs_z {
            (point.z, point.x)
        } else {
            (point.x, point.y)
        }
    }

    fn signed_area_2d(a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> f64 {
        (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
    }

    fn point_in_triangle(p: (f64, f64), a: (f64, f64), b: (f64, f64), c: (f64, f64)) -> bool {
        let d1 = Self::signed_area_2d(p, a, b);
        let d2 = Self::signed_area_2d(p, b, c);
        let d3 = Self::signed_area_2d(p, c, a);
        let has_neg = d1 < 0.0 || d2 < 0.0 || d3 < 0.0;
        let has_pos = d1 > 0.0 || d2 > 0.0 || d3 > 0.0;
        !(has_neg && has_pos)
    }

    fn triangle_area_2x(a: Vec3, b: Vec3, c: Vec3) -> f64 {
        (b - a).cross(c - a).length()
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_obj(content: &[u8]) -> PathBuf {
        let nonce = COUNTER.fetch_add(1, Ordering::Relaxed);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|value| value.as_nanos())
            .unwrap_or(0);
        let path = std::env::temp_dir().join(format!("er_obj_test_{stamp}_{nonce}.obj"));
        let mut file = fs::File::create(&path).expect("create temp file");
        file.write_all(content).expect("write temp file");
        path
    }

    fn parse_str(source: &str) -> Result<(), ObjLoadError> {
        ObjLoader::parse_str(source, "test".to_string()).map(|_| ())
    }

    #[test]
    fn empty_file_returns_error_not_asteroid() {
        let path = temp_obj(b"");
        let result = ObjLoader.load_from_path(&path);
        let _ = fs::remove_file(&path);
        assert!(result.is_err(), "expected error for empty file");
    }

    #[test]
    fn comment_only_file_returns_no_geometry() {
        let result = parse_str("# only comments\n# nothing useful\n");
        assert!(matches!(result, Err(ObjLoadError::NoGeometry)));
    }

    #[test]
    fn nan_position_is_rejected() {
        let result = parse_str("v 1.0 NaN 0.0\nv 0 0 0\nv 1 0 0\nf 1 2 3\n");
        assert!(matches!(result, Err(ObjLoadError::NonFiniteFloat { .. })));
    }

    #[test]
    fn positive_infinity_position_is_rejected() {
        let result = parse_str("v inf 0 0\n");
        assert!(matches!(result, Err(ObjLoadError::NonFiniteFloat { .. })));
    }

    #[test]
    fn negative_infinity_position_is_rejected() {
        let result = parse_str("v 0 0 -inf\n");
        assert!(matches!(result, Err(ObjLoadError::NonFiniteFloat { .. })));
    }

    #[test]
    fn missing_vertex_component_is_rejected() {
        let result = parse_str("v 1.0 2.0\n");
        assert!(matches!(
            result,
            Err(ObjLoadError::MissingComponent { directive: "v", .. })
        ));
    }

    #[test]
    fn invalid_float_token_is_rejected() {
        let result = parse_str("v 1.0 hello 0.0\n");
        assert!(matches!(result, Err(ObjLoadError::InvalidFloat { .. })));
    }

    #[test]
    fn zero_index_is_rejected_per_obj_spec() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 0 1 2\n";
        let result = parse_str(source);
        assert!(matches!(result, Err(ObjLoadError::ZeroIndex { .. })));
    }

    #[test]
    fn positive_index_out_of_range_is_rejected() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 99\n";
        let result = parse_str(source);
        assert!(matches!(result, Err(ObjLoadError::IndexOutOfRange { .. })));
    }

    #[test]
    fn negative_index_out_of_range_is_rejected() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf -1 -2 -99\n";
        let result = parse_str(source);
        assert!(matches!(result, Err(ObjLoadError::IndexOutOfRange { .. })));
    }

    #[test]
    fn negative_index_relative_resolution_works() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n";
        let mesh = ObjLoader::parse_str(source, "rel".to_string()).expect("valid relative");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn malformed_index_token_is_rejected() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 abc\n";
        let result = parse_str(source);
        assert!(matches!(result, Err(ObjLoadError::InvalidIndex { .. })));
    }

    #[test]
    fn face_with_two_vertices_is_rejected() {
        let source = "v 0 0 0\nv 1 0 0\nf 1 2\n";
        let result = parse_str(source);
        assert!(matches!(result, Err(ObjLoadError::FaceTooSmall { .. })));
    }

    #[test]
    fn face_too_large_is_rejected() {
        let mut source = String::new();
        let count = MAX_OBJ_FACE_VERTICES + 1;
        for i in 0..count {
            source.push_str(&format!("v {i} 0 0\n"));
        }
        source.push('f');
        for i in 1..=count {
            source.push_str(&format!(" {i}"));
        }
        source.push('\n');
        let result = parse_str(&source);
        assert!(matches!(result, Err(ObjLoadError::FaceTooLarge { .. })));
    }

    #[test]
    fn crlf_line_endings_are_handled() {
        let source = "v 0 0 0\r\nv 1 0 0\r\nv 0 1 0\r\nf 1 2 3\r\n";
        let mesh = ObjLoader::parse_str(source, "crlf".to_string()).expect("crlf parse");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn utf8_bom_is_skipped() {
        let mut source = String::from("\u{feff}");
        source.push_str("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n");
        let mesh = ObjLoader::parse_str(&source, "bom".to_string()).expect("bom parse");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn comment_after_directive_is_stripped() {
        let source = "v 0 0 0 # origin\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let mesh = ObjLoader::parse_str(source, "cmt".to_string()).expect("comment parse");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn quad_is_triangulated_into_two_triangles() {
        let source = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let mesh = ObjLoader::parse_str(source, "quad".to_string()).expect("quad parse");
        assert_eq!(mesh.indices.len(), 6);
    }

    #[test]
    fn concave_pentagon_emits_only_valid_triangles() {
        let source = "v 0 0 0\nv 2 0 0\nv 1 1 0\nv 2 2 0\nv 0 2 0\nf 1 2 3 4 5\n";
        let mesh = ObjLoader::parse_str(source, "concave".to_string()).expect("concave parse");
        assert_eq!(mesh.indices.len() % 3, 0);
        assert_eq!(mesh.indices.len(), 9);
        for triangle in mesh.indices.chunks(3) {
            let a = mesh.vertices[triangle[0]].position;
            let b = mesh.vertices[triangle[1]].position;
            let c = mesh.vertices[triangle[2]].position;
            let area = (b - a).cross(c - a).length();
            assert!(area > f64::EPSILON, "degenerate triangle emitted");
        }
    }

    #[test]
    fn degenerate_collinear_triangle_is_dropped() {
        let source = "v 0 0 0\nv 1 0 0\nv 2 0 0\nf 1 2 3\n";
        let result = parse_str(source);
        assert!(matches!(result, Err(ObjLoadError::NoGeometry)));
    }

    #[test]
    fn file_too_large_constant_is_positive() {
        assert!(MAX_OBJ_FILE_SIZE > 0);
    }

    #[test]
    fn face_with_double_slash_normal_only_parses() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 1\nf 1//1 2//1 3//1\n";
        let mesh = ObjLoader::parse_str(source, "ns".to_string()).expect("normal-only parse");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn face_with_full_vtn_parses() {
        let source = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 0 1\nvn 0 0 1\nf 1/1/1 2/2/1 3/3/1\n";
        let mesh = ObjLoader::parse_str(source, "full".to_string()).expect("full parse");
        assert_eq!(mesh.indices.len(), 3);
    }

    #[test]
    fn fuzz_random_inputs_never_panic() {
        let mut state: u64 = 0xDEAD_BEEF_CAFE_BABE;
        let alphabet: &[u8] = b"vfntg 0123456789-./\n#\t/inf-NaN";
        for _ in 0..200 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let len = ((state >> 32) as usize) % 4096;
            let mut buffer = Vec::with_capacity(len);
            let mut local = state;
            for _ in 0..len {
                local = local
                    .wrapping_mul(6364136223846793005)
                    .wrapping_add(1442695040888963407);
                let pick = ((local >> 24) as usize) % alphabet.len();
                buffer.push(alphabet[pick]);
            }
            let source = String::from_utf8_lossy(&buffer).to_string();
            let result = ObjLoader::parse_str(&source, "fuzz".to_string());
            if let Ok(mesh) = result {
                assert_eq!(mesh.indices.len() % 3, 0);
                for index in &mesh.indices {
                    assert!(*index < mesh.vertices.len(), "index oob in fuzz output");
                }
                for vertex in &mesh.vertices {
                    assert!(vertex.position.x.is_finite(), "non-finite vertex emitted");
                    assert!(vertex.position.y.is_finite(), "non-finite vertex emitted");
                    assert!(vertex.position.z.is_finite(), "non-finite vertex emitted");
                }
                assert!(mesh.descriptor.bounding_radius > 0.0);
            }
        }
    }
}

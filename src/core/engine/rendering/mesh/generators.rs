//! Procedural mesh generators: asteroid, cube, ground plane, torus,
//! and icosphere.

use std::collections::HashMap;

use crate::core::engine::rendering::raytracing::Vec3;

use super::vertex::{MeshDescriptor, Vertex};
use super::asset::MeshAsset;

impl MeshAsset {
    /// Generates a procedurally displaced asteroid mesh.
    pub fn procedural_asteroid(name: &str, base_radius: f64, subdivisions: u32) -> Self {
        let roughness = 0.25;
        let mut asset = icosphere(subdivisions, base_radius);

        for v in &mut asset.vertices {
            let n = v.position.normalize();
            let noise = crate::core::engine::rendering::utils::fbm_3d(n * 3.0, 4, 2.0, 0.5);
            let displacement = base_radius * roughness * noise;
            v.position = n * (base_radius + displacement);
            v.normal = n;
        }

        asset.name = name.to_string();
        asset.descriptor.bounding_radius = base_radius * (1.0 + roughness);
        asset
    }
}

/// Generates an axis-aligned unit cube (`±0.5`) with outward normals
/// and basic UV coordinates.
pub fn unit_cube() -> MeshAsset {
    let mut vertices = Vec::with_capacity(24);
    let mut indices = Vec::with_capacity(36);

    let faces: &[(Vec3, Vec3, Vec3)] = &[
        (Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(0.0, 0.0, -1.0), Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(-1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 1.0, 0.0)),
        (Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, -1.0)),
        (Vec3::new(0.0, -1.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0)),
    ];

    for &(normal, right, up) in faces {
        let base = vertices.len();
        let center = normal * 0.5;
        vertices.push(Vertex::new(center - right * 0.5 - up * 0.5, normal, Vec3::new(0.0, 0.0, 0.0), Vec3::ZERO));
        vertices.push(Vertex::new(center + right * 0.5 - up * 0.5, normal, Vec3::new(1.0, 0.0, 0.0), Vec3::ZERO));
        vertices.push(Vertex::new(center + right * 0.5 + up * 0.5, normal, Vec3::new(1.0, 1.0, 0.0), Vec3::ZERO));
        vertices.push(Vertex::new(center - right * 0.5 + up * 0.5, normal, Vec3::new(0.0, 1.0, 0.0), Vec3::ZERO));
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    MeshAsset {
        name: "cube".to_string(),
        descriptor: MeshDescriptor {
            vertex_count: vertices.len(),
            triangle_count: indices.len() / 3,
            bounding_radius: 3.0_f64.sqrt() * 0.5,
        },
        vertices,
        indices,
        preferred_material: None,
        base_translation: Vec3::ZERO,
        base_scale: Vec3::ONE,
        base_rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

/// Generates a subdivided ground plane on the XZ plane centred at
/// the origin.
///
/// * `subdivisions` – number of divisions along each axis.
/// * `half_extent`  – half the side length of the plane.
pub fn ground_plane(subdivisions: usize, half_extent: f64) -> MeshAsset {
    let n = subdivisions + 1;
    let mut vertices = Vec::with_capacity(n * n);
    let mut indices = Vec::with_capacity(subdivisions * subdivisions * 6);
    let step = (half_extent * 2.0) / subdivisions as f64;

    for z in 0..n {
        for x in 0..n {
            let px = -half_extent + x as f64 * step;
            let pz = -half_extent + z as f64 * step;
            let u = x as f64 / subdivisions as f64;
            let v = z as f64 / subdivisions as f64;
            vertices.push(Vertex::new(
                Vec3::new(px, 0.0, pz),
                Vec3::new(0.0, 1.0, 0.0),
                Vec3::new(u, v, 0.0),
                Vec3::ZERO,
            ));
        }
    }

    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let i = z * n + x;
            indices.extend_from_slice(&[i, i + n, i + 1, i + 1, i + n, i + n + 1]);
        }
    }

    MeshAsset {
        name: String::new(),
        descriptor: MeshDescriptor {
            vertex_count: vertices.len(),
            triangle_count: indices.len() / 3,
            bounding_radius: 1.0,
        },
        vertices,
        indices,
        preferred_material: None,
        base_translation: Vec3::ZERO,
        base_scale: Vec3::ONE,
        base_rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

/// Generates a torus on the XZ plane.
///
/// * `major_radius` – distance from the centre of the tube to the
///   centre of the torus.
/// * `minor_radius` – radius of the tube.
/// * `major_segments` – subdivisions around the main ring.
/// * `minor_segments` – subdivisions around the tube cross-section.
pub fn torus(
    major_radius: f64,
    minor_radius: f64,
    major_segments: usize,
    minor_segments: usize,
) -> MeshAsset {
    let mut vertices = Vec::with_capacity(major_segments * minor_segments);
    let mut indices = Vec::new();

    for i in 0..major_segments {
        let theta = 2.0 * std::f64::consts::PI * i as f64 / major_segments as f64;
        let (st, ct) = theta.sin_cos();

        for j in 0..minor_segments {
            let phi = 2.0 * std::f64::consts::PI * j as f64 / minor_segments as f64;
            let (sp, cp) = phi.sin_cos();

            let x = (major_radius + minor_radius * cp) * ct;
            let y = minor_radius * sp;
            let z = (major_radius + minor_radius * cp) * st;

            let nx = cp * ct;
            let ny = sp;
            let nz = cp * st;

            let u = i as f64 / major_segments as f64;
            let v = j as f64 / minor_segments as f64;

            vertices.push(Vertex::new(
                Vec3::new(x, y, z),
                Vec3::new(nx, ny, nz),
                Vec3::new(u, v, 0.0),
                Vec3::ZERO,
            ));

            let next_i = (i + 1) % major_segments;
            let next_j = (j + 1) % minor_segments;
            let a = i * minor_segments + j;
            let b = next_i * minor_segments + j;
            let c = next_i * minor_segments + next_j;
            let d = i * minor_segments + next_j;
            indices.extend_from_slice(&[a, b, c, a, c, d]);
        }
    }

    MeshAsset {
        name: String::new(),
        descriptor: MeshDescriptor {
            vertex_count: vertices.len(),
            triangle_count: indices.len() / 3,
            bounding_radius: 1.0,
        },
        vertices,
        indices,
        preferred_material: None,
        base_translation: Vec3::ZERO,
        base_scale: Vec3::ONE,
        base_rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

/// Generates a subdivided icosphere.
///
/// * `subdivisions` – number of recursive subdivisions (0 =
///   icosahedron, each step quadruples the triangle count).
/// * `radius` – sphere radius.
pub fn icosphere(subdivisions: u32, radius: f64) -> MeshAsset {
    let subdivisions = subdivisions.min(8);
    let t = (1.0 + 5.0_f64.sqrt()) / 2.0;

    let raw = [
        Vec3::new(-1.0, t, 0.0),
        Vec3::new(1.0, t, 0.0),
        Vec3::new(-1.0, -t, 0.0),
        Vec3::new(1.0, -t, 0.0),
        Vec3::new(0.0, -1.0, t),
        Vec3::new(0.0, 1.0, t),
        Vec3::new(0.0, -1.0, -t),
        Vec3::new(0.0, 1.0, -t),
        Vec3::new(t, 0.0, -1.0),
        Vec3::new(t, 0.0, 1.0),
        Vec3::new(-t, 0.0, -1.0),
        Vec3::new(-t, 0.0, 1.0),
    ];

    let mut positions: Vec<Vec3> = raw.iter().map(|p| p.normalize()).collect();
    let mut tris: Vec<[usize; 3]> = vec![
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ];

    let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

    for _ in 0..subdivisions {
        let mut new_tris = Vec::with_capacity(tris.len() * 4);
        midpoint_cache.clear();

        for tri in &tris {
            let a = get_midpoint(tri[0], tri[1], &mut positions, &mut midpoint_cache);
            let b = get_midpoint(tri[1], tri[2], &mut positions, &mut midpoint_cache);
            let c = get_midpoint(tri[2], tri[0], &mut positions, &mut midpoint_cache);

            new_tris.push([tri[0], a, c]);
            new_tris.push([tri[1], b, a]);
            new_tris.push([tri[2], c, b]);
            new_tris.push([a, b, c]);
        }

        tris = new_tris;
    }

    let vertices: Vec<Vertex> = positions
        .iter()
        .map(|&p| {
            let n = p.normalize();
            let u = 0.5 + n.z.atan2(n.x) / (2.0 * std::f64::consts::PI);
            let v = 0.5 - n.y.asin() / std::f64::consts::PI;
            Vertex::new(n * radius, n, Vec3::new(u, v, 0.0), Vec3::ZERO)
        })
        .collect();

    let indices: Vec<usize> = tris.iter().flat_map(|t| t.iter().copied()).collect();

    MeshAsset {
        name: String::new(),
        descriptor: MeshDescriptor {
            vertex_count: vertices.len(),
            triangle_count: indices.len() / 3,
            bounding_radius: 1.0,
        },
        vertices,
        indices,
        preferred_material: None,
        base_translation: Vec3::ZERO,
        base_scale: Vec3::ONE,
        base_rotation: [0.0, 0.0, 0.0, 1.0],
    }
}

/// Returns the index of the midpoint between `a` and `b`, creating
/// and caching it if needed.
fn get_midpoint(
    a: usize,
    b: usize,
    positions: &mut Vec<Vec3>,
    cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }
    let mid = ((positions[a] + positions[b]) * 0.5).normalize();
    let idx = positions.len();
    positions.push(mid);
    cache.insert(key, idx);
    idx
}

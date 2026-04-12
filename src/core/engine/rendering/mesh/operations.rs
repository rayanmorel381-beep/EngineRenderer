//! Post-hoc mesh operations: normal recalculation, tangent
//! computation, and subdivision.

use std::collections::HashMap;

use crate::core::engine::rendering::raytracing::Vec3;

use super::asset::MeshAsset;

/// Recomputes smooth normals by averaging face normals weighted by
/// area at each vertex.
///
/// Modifies `asset.vertices[*].normal` in place.
pub fn recalculate_normals(asset: &mut MeshAsset) {
    // Zero out existing normals.
    for v in &mut asset.vertices {
        v.normal = Vec3::ZERO;
    }

    for tri in asset.indices.chunks_exact(3) {
        let a = asset.vertices[tri[0]].position;
        let b = asset.vertices[tri[1]].position;
        let c = asset.vertices[tri[2]].position;

        let face_normal = (b - a).cross(c - a);
        // Weight by area (length of cross product is 2× area).
        asset.vertices[tri[0]].normal += face_normal;
        asset.vertices[tri[1]].normal += face_normal;
        asset.vertices[tri[2]].normal += face_normal;
    }

    for v in &mut asset.vertices {
        let len = v.normal.length();
        if len > 1e-10 {
            v.normal = v.normal * (1.0 / len);
        }
    }
}

/// Computes per-vertex tangent vectors using the Lengyel method
/// (MikkTSpace-compatible direction, without the bi-tangent sign).
///
/// Requires UV coordinates in `vertex.uv` `(u, v, _)`.
pub fn compute_tangents(asset: &mut MeshAsset) {
    for v in &mut asset.vertices {
        v.tangent = Vec3::ZERO;
    }

    for tri in asset.indices.chunks_exact(3) {
        let v0 = &asset.vertices[tri[0]];
        let v1 = &asset.vertices[tri[1]];
        let v2 = &asset.vertices[tri[2]];

        let edge1 = v1.position - v0.position;
        let edge2 = v2.position - v0.position;

        let duv1 = v1.uv - v0.uv;
        let duv2 = v2.uv - v0.uv;

        let denom = duv1.x * duv2.y - duv2.x * duv1.y;
        if denom.abs() < 1e-10 {
            continue;
        }
        let r = 1.0 / denom;

        let tangent = (edge1 * duv2.y - edge2 * duv1.y) * r;

        asset.vertices[tri[0]].tangent += tangent;
        asset.vertices[tri[1]].tangent += tangent;
        asset.vertices[tri[2]].tangent += tangent;
    }

    for v in &mut asset.vertices {
        // Gram–Schmidt orthogonalise against the normal.
        let t = v.tangent - v.normal * v.normal.dot(v.tangent);
        let len = t.length();
        v.tangent = if len > 1e-10 { t * (1.0 / len) } else { Vec3::ZERO };
    }
}

/// Performs one step of Loop subdivision (topology only; does not
/// apply the smoothing weights).
///
/// Each triangle is split into four by inserting midpoints on every
/// edge.
pub fn subdivide(asset: &mut MeshAsset) {
    let mut new_indices: Vec<usize> = Vec::with_capacity(asset.indices.len() * 4);
    let mut midpoint_cache: HashMap<(usize, usize), usize> = HashMap::new();

    for tri in asset.indices.chunks_exact(3) {
        let a = edge_midpoint(tri[0], tri[1], &mut asset.vertices, &mut midpoint_cache);
        let b = edge_midpoint(tri[1], tri[2], &mut asset.vertices, &mut midpoint_cache);
        let c = edge_midpoint(tri[2], tri[0], &mut asset.vertices, &mut midpoint_cache);

        new_indices.extend_from_slice(&[tri[0], a, c]);
        new_indices.extend_from_slice(&[tri[1], b, a]);
        new_indices.extend_from_slice(&[tri[2], c, b]);
        new_indices.extend_from_slice(&[a, b, c]);
    }

    asset.indices = new_indices;
}

/// Returns the vertex index for the midpoint of the edge `(a, b)`,
/// creating a new vertex if one does not yet exist.
fn edge_midpoint(
    a: usize,
    b: usize,
    vertices: &mut Vec<super::vertex::Vertex>,
    cache: &mut HashMap<(usize, usize), usize>,
) -> usize {
    let key = if a < b { (a, b) } else { (b, a) };
    if let Some(&idx) = cache.get(&key) {
        return idx;
    }

    let va = vertices[a];
    let vb = vertices[b];

    let mid = super::vertex::Vertex::new(
        (va.position + vb.position) * 0.5,
        ((va.normal + vb.normal) * 0.5).normalize(),
        (va.uv + vb.uv) * 0.5,
        Vec3::ZERO,
    );

    let idx = vertices.len();
    vertices.push(mid);
    cache.insert(key, idx);
    idx
}

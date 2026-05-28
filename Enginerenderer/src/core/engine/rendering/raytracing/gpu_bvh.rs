//! CPU-side BVH builder + std430 packing for the GPU path tracer.
//!
//! Builds a binary BVH over both spheres and triangles using a top-down
//! median-split on the longest centroid axis. Output is a flat DFS array
//! suitable for stack-based traversal in GLSL.

use super::math::Vec3;
use super::primitives::{Sphere, Triangle};
use super::scene::Scene;

/// One BVH node, std430-aligned (32 bytes).
///
/// Layout in shader: two `vec4`. `leaf_count == 0` means internal node;
/// otherwise it is a leaf containing `leaf_count` consecutive primitive
/// references starting at `right_or_first`.
#[derive(Debug, Clone, Copy)]
pub struct GpuBvhNode {
    pub min: [f32; 3],
    pub leaf_count: u32,
    pub max: [f32; 3],
    pub right_or_first: u32,
}

/// Reference to a single primitive (kind=0 sphere, kind=1 triangle).
#[derive(Debug, Clone, Copy)]
pub struct GpuPrimRef {
    pub kind: u32,
    pub index: u32,
}

/// Output of the BVH build step.
pub struct GpuBvhBuild {
    pub nodes: Vec<GpuBvhNode>,
    pub prim_refs: Vec<GpuPrimRef>,
}

#[derive(Clone, Copy)]
struct PrimAabb {
    kind: u32,
    index: u32,
    centroid: Vec3,
    min: Vec3,
    max: Vec3,
}

const LEAF_THRESHOLD: usize = 4;
const MAX_DEPTH: usize = 32;

/// Builds a BVH over all spheres + triangles in the scene.
pub fn build(scene: &Scene) -> GpuBvhBuild {
    let mut prims: Vec<PrimAabb> = Vec::with_capacity(scene.objects.len() + scene.triangles.len());
    for (i, s) in scene.objects.iter().enumerate() {
        prims.push(sphere_aabb(s, i as u32));
    }
    for (i, t) in scene.triangles.iter().enumerate() {
        prims.push(triangle_aabb(t, i as u32));
    }

    if prims.is_empty() {
        let zero = GpuBvhNode {
            min: [0.0; 3],
            leaf_count: 0,
            max: [0.0; 3],
            right_or_first: 0,
        };
        return GpuBvhBuild {
            nodes: vec![zero],
            prim_refs: vec![GpuPrimRef { kind: 0, index: 0 }],
        };
    }

    let mut nodes: Vec<GpuBvhNode> = Vec::with_capacity(prims.len() * 2);
    let mut prim_refs: Vec<GpuPrimRef> = Vec::with_capacity(prims.len());
    nodes.push(GpuBvhNode {
        min: [0.0; 3],
        leaf_count: 0,
        max: [0.0; 3],
        right_or_first: 0,
    });
    build_recursive(&mut prims, &mut nodes, &mut prim_refs, 0, 0);
    GpuBvhBuild { nodes, prim_refs }
}

fn build_recursive(
    prims: &mut [PrimAabb],
    nodes: &mut Vec<GpuBvhNode>,
    prim_refs: &mut Vec<GpuPrimRef>,
    node_idx: usize,
    depth: usize,
) {
    let (bmin, bmax) = bounds_of(prims);
    if prims.len() <= LEAF_THRESHOLD || depth >= MAX_DEPTH {
        let first = prim_refs.len() as u32;
        for p in prims.iter() {
            prim_refs.push(GpuPrimRef {
                kind: p.kind,
                index: p.index,
            });
        }
        nodes[node_idx] = GpuBvhNode {
            min: [bmin.x as f32, bmin.y as f32, bmin.z as f32],
            leaf_count: prims.len() as u32,
            max: [bmax.x as f32, bmax.y as f32, bmax.z as f32],
            right_or_first: first,
        };
        return;
    }

    let (cmin, cmax) = centroid_bounds(prims);
    let extent = cmax - cmin;
    let axis = if extent.x >= extent.y && extent.x >= extent.z {
        0
    } else if extent.y >= extent.z {
        1
    } else {
        2
    };

    prims.sort_unstable_by(|a, b| {
        axis_value(a.centroid, axis)
            .partial_cmp(&axis_value(b.centroid, axis))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mid = prims.len() / 2;

    let left_idx = nodes.len();
    nodes.push(GpuBvhNode {
        min: [0.0; 3],
        leaf_count: 0,
        max: [0.0; 3],
        right_or_first: 0,
    });
    let right_idx = nodes.len();
    nodes.push(GpuBvhNode {
        min: [0.0; 3],
        leaf_count: 0,
        max: [0.0; 3],
        right_or_first: 0,
    });

    nodes[node_idx] = GpuBvhNode {
        min: [bmin.x as f32, bmin.y as f32, bmin.z as f32],
        leaf_count: 0,
        max: [bmax.x as f32, bmax.y as f32, bmax.z as f32],
        right_or_first: right_idx as u32,
    };

    let (left, right) = prims.split_at_mut(mid);
    build_recursive(left, nodes, prim_refs, left_idx, depth + 1);
    build_recursive(right, nodes, prim_refs, right_idx, depth + 1);
}

fn axis_value(v: Vec3, axis: u8) -> f64 {
    match axis {
        0 => v.x,
        1 => v.y,
        _ => v.z,
    }
}

fn sphere_aabb(s: &Sphere, idx: u32) -> PrimAabb {
    let r = s.radius;
    let r3 = Vec3::new(r, r, r);
    PrimAabb {
        kind: 0,
        index: idx,
        centroid: s.center,
        min: s.center - r3,
        max: s.center + r3,
    }
}

fn triangle_aabb(t: &Triangle, idx: u32) -> PrimAabb {
    let min = vec_min(vec_min(t.a, t.b), t.c);
    let max = vec_max(vec_max(t.a, t.b), t.c);
    let centroid = (t.a + t.b + t.c) * (1.0 / 3.0);
    PrimAabb {
        kind: 1,
        index: idx,
        centroid,
        min,
        max,
    }
}

fn bounds_of(prims: &[PrimAabb]) -> (Vec3, Vec3) {
    let mut bmin = prims[0].min;
    let mut bmax = prims[0].max;
    for p in &prims[1..] {
        bmin = vec_min(bmin, p.min);
        bmax = vec_max(bmax, p.max);
    }
    (bmin, bmax)
}

fn centroid_bounds(prims: &[PrimAabb]) -> (Vec3, Vec3) {
    let mut bmin = prims[0].centroid;
    let mut bmax = prims[0].centroid;
    for p in &prims[1..] {
        bmin = vec_min(bmin, p.centroid);
        bmax = vec_max(bmax, p.centroid);
    }
    (bmin, bmax)
}

fn vec_min(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z))
}

fn vec_max(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z))
}

/// Packs nodes into a single std430 SSBO blob (32 bytes per node).
pub fn pack_nodes(nodes: &[GpuBvhNode]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(nodes.len() * 32);
    for n in nodes {
        bytes.extend_from_slice(&n.min[0].to_le_bytes());
        bytes.extend_from_slice(&n.min[1].to_le_bytes());
        bytes.extend_from_slice(&n.min[2].to_le_bytes());
        bytes.extend_from_slice(&n.leaf_count.to_le_bytes());
        bytes.extend_from_slice(&n.max[0].to_le_bytes());
        bytes.extend_from_slice(&n.max[1].to_le_bytes());
        bytes.extend_from_slice(&n.max[2].to_le_bytes());
        bytes.extend_from_slice(&n.right_or_first.to_le_bytes());
    }
    bytes
}

/// Packs prim refs as `uvec4(kind, index, 0, 0)` (16 bytes each, std430-safe).
pub fn pack_prim_refs(refs: &[GpuPrimRef]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(refs.len() * 16);
    for r in refs {
        bytes.extend_from_slice(&r.kind.to_le_bytes());
        bytes.extend_from_slice(&r.index.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
    }
    bytes
}

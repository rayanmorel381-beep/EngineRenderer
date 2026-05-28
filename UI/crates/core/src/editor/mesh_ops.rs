use crate::scene::mesh::{EditMesh, FaceId, VertId};

fn vadd(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
fn vscale(a: [f64; 3], t: f64) -> [f64; 3] { [a[0]*t, a[1]*t, a[2]*t] }
fn vsub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0]-b[0], a[1]-b[1], a[2]-b[2]] }

pub fn extrude_selected(mesh: &mut EditMesh, offset: f64) {
    let sel_vids: Vec<VertId> = mesh.selected_verts();
    if sel_vids.is_empty() { return; }

    let normal = selected_average_normal(mesh);
    let dir = vscale(normal, offset);

    let mut new_ids: Vec<(VertId, VertId)> = Vec::new();
    for vid in &sel_vids {
        if let Some(pos) = mesh.vert_pos(*vid) {
            let new_id = mesh.add_vert(vadd(pos, dir));
            new_ids.push((*vid, new_id));
        }
    }

    for (old, new) in &new_ids {
        mesh.add_edge(*old, *new);
    }

    for v in &mut mesh.verts {
        v.selected = new_ids.iter().any(|(_, nid)| *nid == v.id);
    }
    for e in &mut mesh.edges {
        e.selected = false;
    }
    for f in &mut mesh.faces {
        f.selected = false;
    }
}

fn selected_average_normal(mesh: &EditMesh) -> [f64; 3] {
    let sel_vids = mesh.selected_verts();
    let mut acc = [0.0f64; 3];
    let mut count = 0usize;
    for fid in mesh.faces.iter().map(|f| f.id).collect::<Vec<_>>() {
        let face = match mesh.faces.iter().find(|f| f.id == fid) {
            Some(f) => f,
            None => continue,
        };
        if face.verts.iter().any(|v| sel_vids.contains(v)) {
            let n = mesh.face_normal(fid);
            acc[0] += n[0]; acc[1] += n[1]; acc[2] += n[2];
            count += 1;
        }
    }
    if count == 0 { return [0.0, 1.0, 0.0]; }
    let len = (acc[0]*acc[0]+acc[1]*acc[1]+acc[2]*acc[2]).sqrt();
    if len < 1e-12 { [0.0, 1.0, 0.0] } else { [acc[0]/len, acc[1]/len, acc[2]/len] }
}

pub fn subdivide(mesh: &mut EditMesh) {
    let edges_snapshot: Vec<_> = mesh.edges.iter().map(|e| (e.id, e.verts)).collect();
    let faces_snapshot: Vec<_> = mesh.faces.iter().map(|f| (f.id, f.verts.clone())).collect();

    for (_eid, [a, b]) in &edges_snapshot {
        let pa = mesh.vert_pos(*a).unwrap_or([0.0; 3]);
        let pb = mesh.vert_pos(*b).unwrap_or([0.0; 3]);
        let mid = vscale(vadd(pa, pb), 0.5);
        mesh.add_vert(mid);
    }

    let mid_start = mesh.verts.len() - edges_snapshot.len();
    for (face_idx, (_fid, face_verts)) in faces_snapshot.iter().enumerate() {
        let n = face_verts.len();
        let center = {
            let sum = face_verts.iter().fold([0.0f64; 3], |acc, vid| {
                let p = mesh.vert_pos(*vid).unwrap_or([0.0; 3]);
                vadd(acc, p)
            });
            vscale(sum, 1.0 / n as f64)
        };
        let center_id = mesh.add_vert(center);
        let _ = (face_idx, center_id);
    }

    for (i, (_eid, [a, b])) in edges_snapshot.iter().enumerate() {
        let mid_id = mesh.verts[mid_start + i].id;
        mesh.add_edge(*a, mid_id);
        mesh.add_edge(mid_id, *b);
    }

    mesh.edges.retain(|e| {
        !edges_snapshot.iter().any(|(eid, _)| *eid == e.id)
    });
}

pub fn loop_cut(mesh: &mut EditMesh, edge_ring: &[crate::scene::mesh::EdgeId], t: f64) {
    let t = t.clamp(0.0, 1.0);
    for eid in edge_ring {
        let edge = match mesh.edges.iter().find(|e| e.id == *eid) {
            Some(e) => e.clone(),
            None => continue,
        };
        let pa = mesh.vert_pos(edge.verts[0]).unwrap_or([0.0; 3]);
        let pb = mesh.vert_pos(edge.verts[1]).unwrap_or([0.0; 3]);
        let cut_pos = vadd(vscale(pa, 1.0 - t), vscale(pb, t));
        let cut_id = mesh.add_vert(cut_pos);
        mesh.add_edge(edge.verts[0], cut_id);
        mesh.add_edge(cut_id, edge.verts[1]);
    }
    let ring_set: std::collections::HashSet<_> = edge_ring.iter().copied().collect();
    mesh.edges.retain(|e| !ring_set.contains(&e.id));
}

pub fn bevel_selected_edges(mesh: &mut EditMesh, amount: f64) {
    let sel_eids: Vec<_> = mesh.selected_edges();
    for eid in sel_eids {
        let edge = match mesh.edges.iter().find(|e| e.id == eid) {
            Some(e) => e.clone(),
            None => continue,
        };
        let pa = mesh.vert_pos(edge.verts[0]).unwrap_or([0.0; 3]);
        let pb = mesh.vert_pos(edge.verts[1]).unwrap_or([0.0; 3]);
        let dir = vsub(pb, pa);
        let a1 = vadd(pa, vscale(dir, amount));
        let b1 = vadd(pb, vscale(dir, -amount));
        let na = mesh.add_vert(a1);
        let nb = mesh.add_vert(b1);
        mesh.add_edge(na, nb);
        mesh.add_edge(edge.verts[0], na);
        mesh.add_edge(nb, edge.verts[1]);
        mesh.edges.retain(|e| e.id != eid);
    }
}

pub fn inset_face(mesh: &mut EditMesh, face_id: FaceId, amount: f64) {
    let face = match mesh.faces.iter().find(|f| f.id == face_id) {
        Some(f) => f.clone(),
        None => return,
    };
    let center = {
        let sum = face.verts.iter().fold([0.0f64; 3], |acc, vid| {
            vadd(acc, mesh.vert_pos(*vid).unwrap_or([0.0; 3]))
        });
        vscale(sum, 1.0 / face.verts.len() as f64)
    };
    let inner: Vec<VertId> = face.verts.iter().map(|vid| {
        let p = mesh.vert_pos(*vid).unwrap_or([0.0; 3]);
        let inset_pos = vadd(p, vscale(vsub(center, p), amount.clamp(0.0, 0.99)));
        mesh.add_vert(inset_pos)
    }).collect();
    let n = inner.len();
    for i in 0..n {
        mesh.add_edge(inner[i], inner[(i + 1) % n]);
        mesh.add_edge(face.verts[i], inner[i]);
        mesh.add_face(vec![face.verts[i], face.verts[(i+1)%n], inner[(i+1)%n], inner[i]]);
    }
    mesh.add_face(inner.clone());
    mesh.faces.retain(|f| f.id != face_id);
}

pub fn merge_vertices_by_distance(mesh: &mut EditMesh, threshold: f64) {
    let threshold_sq = threshold * threshold;
    let n = mesh.verts.len();
    let mut remap: Vec<usize> = (0..n).collect();
    for i in 0..n {
        for j in (i + 1)..n {
            if remap[j] != j { continue; }
            let pi = mesh.verts[i].pos;
            let pj = mesh.verts[j].pos;
            let d = vsub(pi, pj);
            if d[0]*d[0]+d[1]*d[1]+d[2]*d[2] < threshold_sq {
                remap[j] = i;
            }
        }
    }
    for edge in &mut mesh.edges {
        let ia = mesh.verts.iter().position(|v| v.id == edge.verts[0]).unwrap_or(0);
        let ib = mesh.verts.iter().position(|v| v.id == edge.verts[1]).unwrap_or(0);
        let ra = mesh.verts[remap[ia]].id;
        let rb = mesh.verts[remap[ib]].id;
        edge.verts = [ra, rb];
    }
    mesh.edges.retain(|e| e.verts[0] != e.verts[1]);
    for face in &mut mesh.faces {
        let remapped: Vec<VertId> = face.verts.iter().map(|vid| {
            let idx = mesh.verts.iter().position(|v| v.id == *vid).unwrap_or(0);
            mesh.verts[remap[idx]].id
        }).collect();
        face.verts = remapped;
    }
    let keep: Vec<bool> = (0..n).map(|i| remap[i] == i).collect();
    let mut ki = 0usize;
    mesh.verts.retain(|_| { let r = keep[ki]; ki += 1; r });
}

pub fn flip_face_normals(mesh: &mut EditMesh) {
    for face in &mut mesh.faces {
        face.verts.reverse();
    }
}

pub fn select_ring(mesh: &EditMesh, start: crate::scene::mesh::EdgeId) -> Vec<crate::scene::mesh::EdgeId> {
    let start_edge = match mesh.edges.iter().find(|e| e.id == start) {
        Some(e) => e.clone(),
        None => return Vec::new(),
    };
    let mut ring = vec![start_edge.id];
    let [mut va, mut vb] = start_edge.verts;
    let max_iter = mesh.edges.len() + 1;
    for _ in 0..max_iter {
        let faces_a = mesh.faces_of_vert(va);
        let faces_b = mesh.faces_of_vert(vb);
        let shared: Vec<_> = faces_a.iter().filter(|f| faces_b.contains(f)).copied().collect();
        let Some(face_id) = shared.into_iter().find(|fid| {
            mesh.edges_of_face(*fid).iter().any(|eid| *eid == *ring.last().unwrap())
        }) else {
            break;
        };
        let face = match mesh.faces.iter().find(|f| f.id == face_id) {
            Some(f) => f,
            None => break,
        };
        let n = face.verts.len();
        let idx_va = face.verts.iter().position(|v| *v == va).unwrap_or(0);
        let opposite_va = face.verts[(idx_va + n / 2) % n];
        let idx_vb = face.verts.iter().position(|v| *v == vb).unwrap_or(0);
        let opposite_vb = face.verts[(idx_vb + n / 2) % n];
        let next_edge = mesh.edges.iter().find(|e| {
            (e.verts[0] == opposite_va && e.verts[1] == opposite_vb)
            || (e.verts[0] == opposite_vb && e.verts[1] == opposite_va)
        });
        match next_edge {
            Some(e) if !ring.contains(&e.id) => {
                ring.push(e.id);
                va = opposite_va;
                vb = opposite_vb;
            }
            _ => break,
        }
    }
    ring
}

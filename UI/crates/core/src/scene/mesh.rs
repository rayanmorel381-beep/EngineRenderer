use crate::scene::ObjectId;

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct VertId(pub u32);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct EdgeId(pub u32);

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct FaceId(pub u32);

#[derive(Clone, Debug)]
pub struct Vert {
    pub id: VertId,
    pub pos: [f64; 3],
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct MeshEdge {
    pub id: EdgeId,
    pub verts: [VertId; 2],
    pub selected: bool,
}

#[derive(Clone, Debug)]
pub struct Face {
    pub id: FaceId,
    pub verts: Vec<VertId>,
    pub selected: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SelectMode {
    Vertex,
    Edge,
    Face,
}

impl Default for SelectMode {
    fn default() -> Self {
        Self::Vertex
    }
}

#[derive(Clone, Debug)]
pub struct EditMesh {
    pub object_id: ObjectId,
    pub verts: Vec<Vert>,
    pub edges: Vec<MeshEdge>,
    pub faces: Vec<Face>,
    pub select_mode: SelectMode,
    next_vert: u32,
    next_edge: u32,
    next_face: u32,
}

impl EditMesh {
    pub fn new(object_id: ObjectId) -> Self {
        Self {
            object_id,
            verts: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            select_mode: SelectMode::Vertex,
            next_vert: 0,
            next_edge: 0,
            next_face: 0,
        }
    }

    pub fn from_cube(object_id: ObjectId) -> Self {
        let mut mesh = Self::new(object_id);
        let positions: [[f64; 3]; 8] = [
            [-0.5, -0.5, -0.5], [0.5, -0.5, -0.5], [0.5,  0.5, -0.5], [-0.5,  0.5, -0.5],
            [-0.5, -0.5,  0.5], [0.5, -0.5,  0.5], [0.5,  0.5,  0.5], [-0.5,  0.5,  0.5],
        ];
        for pos in positions {
            mesh.add_vert(pos);
        }
        let edge_pairs = [
            (0,1),(1,2),(2,3),(3,0),
            (4,5),(5,6),(6,7),(7,4),
            (0,4),(1,5),(2,6),(3,7),
        ];
        for (a, b) in edge_pairs {
            mesh.add_edge(VertId(a), VertId(b));
        }
        let face_quads = [
            [0,1,2,3],[4,5,6,7],[0,1,5,4],
            [2,3,7,6],[1,2,6,5],[0,3,7,4],
        ];
        for quad in face_quads {
            mesh.add_face(quad.iter().map(|&i| VertId(i)).collect());
        }
        mesh
    }

    pub fn from_plane(object_id: ObjectId) -> Self {
        let mut mesh = Self::new(object_id);
        let positions: [[f64; 3]; 4] = [
            [-0.5, 0.0, -0.5], [0.5, 0.0, -0.5],
            [0.5, 0.0, 0.5], [-0.5, 0.0, 0.5],
        ];
        for pos in positions {
            mesh.add_vert(pos);
        }
        mesh.add_edge(VertId(0), VertId(1));
        mesh.add_edge(VertId(1), VertId(2));
        mesh.add_edge(VertId(2), VertId(3));
        mesh.add_edge(VertId(3), VertId(0));
        mesh.add_face(vec![VertId(0), VertId(1), VertId(2), VertId(3)]);
        mesh
    }

    pub fn add_vert(&mut self, pos: [f64; 3]) -> VertId {
        let id = VertId(self.next_vert);
        self.next_vert += 1;
        self.verts.push(Vert { id, pos, selected: false });
        id
    }

    pub fn add_edge(&mut self, a: VertId, b: VertId) -> EdgeId {
        let id = EdgeId(self.next_edge);
        self.next_edge += 1;
        self.edges.push(MeshEdge { id, verts: [a, b], selected: false });
        id
    }

    pub fn add_face(&mut self, verts: Vec<VertId>) -> FaceId {
        let id = FaceId(self.next_face);
        self.next_face += 1;
        self.faces.push(Face { id, verts, selected: false });
        id
    }

    pub fn select_all(&mut self) {
        for v in &mut self.verts { v.selected = true; }
        for e in &mut self.edges { e.selected = true; }
        for f in &mut self.faces { f.selected = true; }
    }

    pub fn deselect_all(&mut self) {
        for v in &mut self.verts { v.selected = false; }
        for e in &mut self.edges { e.selected = false; }
        for f in &mut self.faces { f.selected = false; }
    }

    pub fn selected_verts(&self) -> Vec<VertId> {
        self.verts.iter().filter(|v| v.selected).map(|v| v.id).collect()
    }

    pub fn selected_edges(&self) -> Vec<EdgeId> {
        self.edges.iter().filter(|e| e.selected).map(|e| e.id).collect()
    }

    pub fn selected_faces(&self) -> Vec<FaceId> {
        self.faces.iter().filter(|f| f.selected).map(|f| f.id).collect()
    }

    pub fn vert_pos(&self, id: VertId) -> Option<[f64; 3]> {
        self.verts.iter().find(|v| v.id == id).map(|v| v.pos)
    }

    pub fn vert_pos_mut(&mut self, id: VertId) -> Option<&mut [f64; 3]> {
        self.verts.iter_mut().find(|v| v.id == id).map(|v| &mut v.pos)
    }

    pub fn centroid(&self) -> [f64; 3] {
        if self.verts.is_empty() { return [0.0; 3]; }
        let sum = self.verts.iter().fold([0.0; 3], |acc, v| {
            [acc[0]+v.pos[0], acc[1]+v.pos[1], acc[2]+v.pos[2]]
        });
        let n = self.verts.len() as f64;
        [sum[0]/n, sum[1]/n, sum[2]/n]
    }

    pub fn selected_centroid(&self) -> Option<[f64; 3]> {
        let sel: Vec<_> = self.verts.iter().filter(|v| v.selected).collect();
        if sel.is_empty() { return None; }
        let sum = sel.iter().fold([0.0; 3], |acc, v| {
            [acc[0]+v.pos[0], acc[1]+v.pos[1], acc[2]+v.pos[2]]
        });
        let n = sel.len() as f64;
        Some([sum[0]/n, sum[1]/n, sum[2]/n])
    }

    pub fn edges_of_face(&self, face_id: FaceId) -> Vec<EdgeId> {
        let face = match self.faces.iter().find(|f| f.id == face_id) {
            Some(f) => f,
            None => return Vec::new(),
        };
        let n = face.verts.len();
        let mut result = Vec::new();
        for i in 0..n {
            let a = face.verts[i];
            let b = face.verts[(i + 1) % n];
            if let Some(e) = self.edges.iter().find(|e| {
                (e.verts[0] == a && e.verts[1] == b) || (e.verts[0] == b && e.verts[1] == a)
            }) {
                result.push(e.id);
            }
        }
        result
    }

    pub fn faces_of_vert(&self, vert_id: VertId) -> Vec<FaceId> {
        self.faces.iter().filter(|f| f.verts.contains(&vert_id)).map(|f| f.id).collect()
    }

    pub fn faces_of_edge(&self, edge_id: EdgeId) -> Vec<FaceId> {
        let edge = match self.edges.iter().find(|e| e.id == edge_id) {
            Some(e) => e,
            None => return Vec::new(),
        };
        let [a, b] = edge.verts;
        self.faces.iter()
            .filter(|f| f.verts.contains(&a) && f.verts.contains(&b))
            .map(|f| f.id)
            .collect()
    }

    pub fn face_normal(&self, face_id: FaceId) -> [f64; 3] {
        let face = match self.faces.iter().find(|f| f.id == face_id) {
            Some(f) => f,
            None => return [0.0, 1.0, 0.0],
        };
        if face.verts.len() < 3 { return [0.0, 1.0, 0.0]; }
        let p0 = self.vert_pos(face.verts[0]).unwrap_or([0.0; 3]);
        let p1 = self.vert_pos(face.verts[1]).unwrap_or([0.0; 3]);
        let p2 = self.vert_pos(face.verts[2]).unwrap_or([0.0; 3]);
        let u = [p1[0]-p0[0], p1[1]-p0[1], p1[2]-p0[2]];
        let v = [p2[0]-p0[0], p2[1]-p0[1], p2[2]-p0[2]];
        let n = [u[1]*v[2]-u[2]*v[1], u[2]*v[0]-u[0]*v[2], u[0]*v[1]-u[1]*v[0]];
        let len = (n[0]*n[0]+n[1]*n[1]+n[2]*n[2]).sqrt();
        if len < 1e-12 { [0.0, 1.0, 0.0] } else { [n[0]/len, n[1]/len, n[2]/len] }
    }
}

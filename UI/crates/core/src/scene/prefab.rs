use crate::scene::{ObjectKind, SceneObject};

#[derive(Clone, Debug)]
pub struct PrefabObject {
    pub name: String,
    pub kind: ObjectKind,
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
    pub intensity: f64,
    pub children: Vec<PrefabObject>,
}

impl PrefabObject {
    pub fn from_scene_object(obj: &SceneObject) -> Self {
        Self {
            name: obj.name.clone(),
            kind: obj.kind.clone(),
            position: obj.position,
            rotation: obj.rotation,
            scale: obj.scale,
            intensity: obj.intensity,
            children: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Prefab {
    pub name: String,
    pub root: PrefabObject,
    pub description: String,
}

impl Prefab {
    pub fn new(name: impl Into<String>, root: PrefabObject) -> Self {
        Self { name: name.into(), root, description: String::new() }
    }

    pub fn from_scene_object(name: impl Into<String>, obj: &SceneObject) -> Self {
        Self::new(name, PrefabObject::from_scene_object(obj))
    }
}

#[derive(Clone, Debug, Default)]
pub struct PrefabLibrary {
    pub prefabs: Vec<Prefab>,
}

impl PrefabLibrary {
    pub fn new() -> Self { Self::default() }

    pub fn add(&mut self, prefab: Prefab) -> usize {
        let idx = self.prefabs.len();
        self.prefabs.push(prefab);
        idx
    }

    pub fn get(&self, idx: usize) -> Option<&Prefab> { self.prefabs.get(idx) }
    pub fn len(&self) -> usize { self.prefabs.len() }
    pub fn is_empty(&self) -> bool { self.prefabs.is_empty() }
}

pub struct PrefabInstance {
    pub prefab_index: usize,
    pub override_position: Option<[f64; 3]>,
    pub override_rotation: Option<[f64; 3]>,
    pub override_scale: Option<[f64; 3]>,
}

impl PrefabInstance {
    pub fn new(prefab_index: usize) -> Self {
        Self { prefab_index, override_position: None, override_rotation: None, override_scale: None }
    }

    pub fn instantiate(&self, library: &PrefabLibrary, scene: &mut crate::scene::Scene) -> Option<crate::scene::ObjectId> {
        let prefab = library.get(self.prefab_index)?;
        let kind = prefab.root.kind.clone();
        let id = match &kind {
            ObjectKind::Primitive(k) => scene.add_primitive(*k),
            ObjectKind::Mesh { asset_index } => scene.add_mesh(prefab.root.name.clone(), *asset_index),
        };
        if let Some(obj) = scene.get_mut(id) {
            obj.position = self.override_position.unwrap_or(prefab.root.position);
            obj.rotation = self.override_rotation.unwrap_or(prefab.root.rotation);
            obj.scale = self.override_scale.unwrap_or(prefab.root.scale);
            obj.intensity = prefab.root.intensity;
        }
        Some(id)
    }
}

use crate::scene::{ObjectId, Scene, SceneObject};

#[derive(Clone, Debug)]
pub struct Transform {
    pub position: [f64; 3],
    pub rotation: [f64; 3],
    pub scale: [f64; 3],
}

impl Transform {
    pub fn from_object(obj: &SceneObject) -> Self {
        Self {
            position: obj.position,
            rotation: obj.rotation,
            scale: obj.scale,
        }
    }
}

#[derive(Clone, Debug)]
pub enum HistoryEntry {
    AddObject(SceneObject),
    RemoveObject(SceneObject),
    SetTransform {
        id: ObjectId,
        before: Transform,
        after: Transform,
    },
}

#[derive(Default)]
pub struct HistoryStack {
    undo: Vec<HistoryEntry>,
    redo: Vec<HistoryEntry>,
}

impl HistoryStack {
    pub fn push(&mut self, entry: HistoryEntry) {
        self.undo.push(entry);
        self.redo.clear();
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn undo(&mut self, scene: &mut Scene) -> Option<Option<ObjectId>> {
        let entry = self.undo.pop()?;
        let result = match &entry {
            HistoryEntry::AddObject(obj) => {
                scene.remove(obj.id);
                Some(None)
            }
            HistoryEntry::RemoveObject(obj) => {
                let id = obj.id;
                scene.restore(obj.clone());
                Some(Some(id))
            }
            HistoryEntry::SetTransform { id, before, .. } => {
                if let Some(obj) = scene.get_mut(*id) {
                    obj.position = before.position;
                    obj.rotation = before.rotation;
                    obj.scale = before.scale;
                }
                Some(Some(*id))
            }
        };
        self.redo.push(entry);
        result
    }

    pub fn redo(&mut self, scene: &mut Scene) -> Option<Option<ObjectId>> {
        let entry = self.redo.pop()?;
        let result = match &entry {
            HistoryEntry::AddObject(obj) => {
                let id = obj.id;
                scene.restore(obj.clone());
                Some(Some(id))
            }
            HistoryEntry::RemoveObject(obj) => {
                scene.remove(obj.id);
                Some(None)
            }
            HistoryEntry::SetTransform { id, after, .. } => {
                if let Some(obj) = scene.get_mut(*id) {
                    obj.position = after.position;
                    obj.rotation = after.rotation;
                    obj.scale = after.scale;
                }
                Some(Some(*id))
            }
        };
        self.undo.push(entry);
        result
    }
}

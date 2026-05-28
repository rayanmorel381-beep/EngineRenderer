use std::any::{Any, TypeId};
use std::collections::HashMap;
use super::entity::Entity;

pub trait AnyStorage: Any + Send + Sync {
    fn remove_entity(&mut self, entity_index: u32);
    fn component_count(&self) -> usize;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

pub struct TypedStorage<T: Any + Send + Sync> {
    sparse: HashMap<u32, usize>,
    entities: Vec<Entity>,
    data: Vec<T>,
}

impl<T: Any + Send + Sync> TypedStorage<T> {
    pub fn new() -> Self {
        Self {
            sparse: HashMap::new(),
            entities: Vec::new(),
            data: Vec::new(),
        }
    }

    pub fn insert(&mut self, entity: Entity, component: T) {
        let index = entity.index();
        if let Some(&dense) = self.sparse.get(&index) {
            self.entities[dense] = entity;
            self.data[dense] = component;
        } else {
            let dense = self.data.len();
            self.sparse.insert(index, dense);
            self.entities.push(entity);
            self.data.push(component);
        }
    }

    pub fn get(&self, entity_index: u32) -> Option<&T> {
        let dense = *self.sparse.get(&entity_index)?;
        Some(&self.data[dense])
    }

    pub fn get_mut(&mut self, entity_index: u32) -> Option<&mut T> {
        let dense = *self.sparse.get(&entity_index)?;
        Some(&mut self.data[dense])
    }

    fn remove_by_index(&mut self, entity_index: u32) {
        if let Some(dense) = self.sparse.remove(&entity_index) {
            let last = self.entities.len() - 1;
            if dense != last {
                let swapped = self.entities[last];
                self.entities.swap(dense, last);
                self.data.swap(dense, last);
                self.sparse.insert(swapped.index(), dense);
            }
            self.entities.pop();
            self.data.pop();
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.entities.iter().copied().zip(self.data.iter())
    }

    pub fn component_count(&self) -> usize {
        self.data.len()
    }
}

impl<T: Any + Send + Sync> AnyStorage for TypedStorage<T> {
    fn remove_entity(&mut self, entity_index: u32) {
        self.remove_by_index(entity_index);
    }

    fn component_count(&self) -> usize {
        TypedStorage::component_count(self)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

pub struct ComponentRegistry {
    storages: HashMap<TypeId, Box<dyn AnyStorage>>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self { storages: HashMap::new() }
    }

    fn get_or_create_typed<T: Any + Send + Sync>(&mut self) -> &mut TypedStorage<T> {
        self.storages
            .entry(TypeId::of::<T>())
            .or_insert_with(|| Box::new(TypedStorage::<T>::new()))
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
            .unwrap()
    }

    fn get_typed<T: Any + Send + Sync>(&self) -> Option<&TypedStorage<T>> {
        self.storages
            .get(&TypeId::of::<T>())?
            .as_any()
            .downcast_ref::<TypedStorage<T>>()
    }

    fn get_typed_mut<T: Any + Send + Sync>(&mut self) -> Option<&mut TypedStorage<T>> {
        self.storages
            .get_mut(&TypeId::of::<T>())?
            .as_any_mut()
            .downcast_mut::<TypedStorage<T>>()
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, entity: Entity, component: T) {
        self.get_or_create_typed::<T>().insert(entity, component);
    }

    pub fn get<T: Any + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        self.get_typed::<T>()?.get(entity.index())
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        self.get_typed_mut::<T>()?.get_mut(entity.index())
    }

    pub fn remove_all(&mut self, entity: Entity) {
        let index = entity.index();
        for storage in self.storages.values_mut() {
            storage.remove_entity(index);
        }
    }

    pub fn query<T: Any + Send + Sync>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.get_typed::<T>()
            .map(|s| s.iter())
            .into_iter()
            .flatten()
    }

    pub fn total_component_count(&self) -> usize {
        self.storages.values().map(|s| s.component_count()).sum()
    }
}

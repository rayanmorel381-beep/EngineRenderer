use super::component::ComponentRegistry;
use super::entity::{Entity, EntityAllocator};
use std::any::{Any, TypeId};

pub struct World {
    entities: EntityAllocator,
    components: ComponentRegistry,
}

impl World {
    pub fn new() -> Self {
        Self {
            entities: EntityAllocator::new(),
            components: ComponentRegistry::new(),
        }
    }

    pub fn spawn(&mut self) -> Entity {
        self.entities.allocate()
    }

    pub fn despawn(&mut self, entity: Entity) {
        self.components.remove_all(entity);
        self.entities.free_entity(entity);
    }

    pub fn insert<T: Any + Send + Sync>(&mut self, entity: Entity, component: T) {
        self.components.insert(entity, component);
    }

    pub fn get<T: Any + Send + Sync>(&self, entity: Entity) -> Option<&T> {
        self.components.get(entity)
    }

    pub fn get_mut<T: Any + Send + Sync>(&mut self, entity: Entity) -> Option<&mut T> {
        self.components.get_mut(entity)
    }

    pub fn query<T: Any + Send + Sync>(&self) -> impl Iterator<Item = (Entity, &T)> {
        self.components.query::<T>()
    }

    pub fn entity_count(&self) -> usize {
        self.entities.alive_count()
    }

    pub fn component_type_id_of<T: Any>() -> TypeId {
        TypeId::of::<T>()
    }

    pub fn is_entity_alive(&self, entity: Entity) -> bool {
        self.entities.is_alive(entity)
    }

    pub fn component_count(&self) -> usize {
        self.components.total_component_count()
    }
}

impl std::fmt::Debug for World {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("World")
            .field("entity_count", &self.entities.alive_count())
            .finish()
    }
}

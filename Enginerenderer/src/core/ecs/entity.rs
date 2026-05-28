/// Entity handle: lower 32 bits = slot index, upper 32 bits = generation.
/// Stale handles from before a despawn have a mismatched generation and are
/// correctly rejected by `is_alive`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Entity(pub u64);

impl Entity {
    pub fn new(index: u32, generation: u32) -> Self {
        Self((generation as u64) << 32 | index as u64)
    }

    pub fn index(self) -> u32 {
        self.0 as u32
    }

    pub fn generation(self) -> u32 {
        (self.0 >> 32) as u32
    }
}

pub struct EntityAllocator {
    generations: Vec<u32>,
    free: Vec<u32>,
}

impl EntityAllocator {
    pub fn new() -> Self {
        Self {
            generations: vec![0],
            free: Vec::new(),
        }
    }

    pub fn allocate(&mut self) -> Entity {
        if let Some(index) = self.free.pop() {
            Entity::new(index, self.generations[index as usize])
        } else {
            let index = self.generations.len() as u32;
            self.generations.push(0);
            Entity::new(index, 0)
        }
    }

    pub fn free_entity(&mut self, entity: Entity) {
        let index = entity.index() as usize;
        if index < self.generations.len() && self.generations[index] == entity.generation() {
            self.generations[index] = self.generations[index].wrapping_add(1);
            self.free.push(entity.index());
        }
    }

    pub fn is_alive(&self, entity: Entity) -> bool {
        let index = entity.index() as usize;
        index > 0
            && index < self.generations.len()
            && self.generations[index] == entity.generation()
    }

    pub fn alive_count(&self) -> usize {
        self.generations.len().saturating_sub(1 + self.free.len())
    }
}

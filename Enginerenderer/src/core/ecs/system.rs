use super::world::World;

pub trait System: Send {
    fn run(&mut self, world: &mut World);
}

pub struct SystemScheduler {
    systems: Vec<Box<dyn System>>,
}

impl SystemScheduler {
    pub fn new() -> Self {
        Self {
            systems: Vec::new(),
        }
    }

    pub fn add_system<S: System + 'static>(&mut self, system: S) {
        self.systems.push(Box::new(system));
    }

    pub fn run_all(&mut self, world: &mut World) {
        for system in &mut self.systems {
            system.run(world);
        }
    }

    pub fn system_count(&self) -> usize {
        self.systems.len()
    }
}

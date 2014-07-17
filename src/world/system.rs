use std::collections::hashmap::{HashMap};
use world::terrain::TerrainType;
use world::spatial::{SpatialSystem, Location};
use calx::world;

pub type Entity = world::Entity<System>;
pub type World = world::World<System>;

pub struct System {
    world: Option<World>,
    pub seed: u32,
    tick: u64,
    pub depth: int,
    pub area: HashMap<Location, TerrainType>,
    pub spatial: SpatialSystem,
}

impl world::System for System {
    fn register(&mut self, world: &World) {
        self.world = Some(world.clone());
    }

    fn added(&mut self, _e: &Entity) {}
    fn changed<C>(&mut self, _e: &Entity, _component: Option<&C>) {}

    fn deleted(&mut self, e: &Entity) {
         self.spatial.remove(e);
    }
}

impl System {
    pub fn new(seed: u32) -> System {
        System {
            world: None,
            seed: seed,
            tick: 0,
            depth: 0,
            area: HashMap::new(),
            spatial: SpatialSystem::new(),
        }
    }
}

pub trait EngineLogic {
    /// Get the number of the current time frame.
    fn get_tick(&self) -> u64;
    /// Advance to the next time frame.
    fn advance_frame(&mut self);
}

impl EngineLogic for World {
    fn get_tick(&self) -> u64 { self.system().tick }

    fn advance_frame(&mut self) { self.system_mut().tick += 1; }
}

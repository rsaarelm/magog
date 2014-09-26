use std::mem;
use std::collections::hashmap::{HashMap};
use world::terrain::TerrainType;
use world::spatial::{SpatialSystem, Location, Position};
use world::fov::Fov;
use world::world;

// XXX: Reference to view layer. Wanted to make this work using traits, but
// the ownership system got too hard to track when I also didn't want to make
// System type-parametrized to keep the Entity and World type aliases simple.
use view::gamestate::Fx;

pub type Entity = world::Entity<System>;
pub type World<'a> = world::World<'a, System>;

/// Global game state values. The entity component system part is handled by
/// the engine code.
pub struct System<'a> {
    world: Option<World<'a>>,
    pub seed: u32,
    tick: u64,
    pub depth: int,
    pub area: HashMap<Location, TerrainType>,
    pub spatial: SpatialSystem,
    pub fx: Fx,
    pub camera: Option<Entity>,
}

impl<'a> world::System for System<'a> {
    fn register(&mut self, world: &World) { self.world = Some(world.clone()); }

    fn added(&mut self, _e: &Entity) {}
    fn changed<C>(&mut self, _e: &Entity, _component: Option<&C>) {}

    fn deleted(&mut self, e: &Entity) {
         self.spatial.remove(e);
    }
}

impl<'a> System<'a> {
    pub fn new(seed: u32, fx: Fx) -> System<'a> {
        System {
            world: None,
            seed: seed,
            tick: 0,
            depth: 0,
            area: HashMap::new(),
            spatial: SpatialSystem::new(),
            fx: fx,
            camera: None,
        }
    }
}

pub trait EngineLogic {
    /// Get the number of the current time frame.
    fn get_tick(&self) -> u64;
    /// Advance to the next time frame.
    fn advance_frame(&mut self);
    /// Get the camera entity
    fn camera(&self) -> Entity;
}

impl EngineLogic for World {
    fn get_tick(&self) -> u64 { self.system().tick }

    fn advance_frame(&mut self) { self.system_mut().tick += 1; }

    fn camera(&self) -> Entity {
        // hacketyhack
        unsafe {
            if self.system().camera.is_none() {
                let mself = mem::transmute::<&World, &mut World>(self);
                let mut camera = mself.new_entity();
                camera.set_component(Fov::new());
                camera.set_location(Location::new(0, 0));

                mself.system_mut().camera = Some(camera);
            }
        }

        self.system().camera.as_ref().unwrap().clone()
    }
}

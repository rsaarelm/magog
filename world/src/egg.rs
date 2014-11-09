use location::{Location};
use {EntityKind, MobKind};
use mob::Mob;
use mob;
use world;
use entity::{Entity};
use map_memory::{MapMemory};

pub struct Egg {
    kind: EntityKind,
}

impl Egg {
    pub fn new(kind: EntityKind) -> Egg {
        Egg {
            kind: kind,
        }
    }

    pub fn hatch(&self, loc: Location) -> Entity {
        let entity = world::get().borrow_mut().ecs.new_entity();

        world::get().borrow_mut().comp.kind.insert(entity, self.kind);

        match self.kind {
            MobKind(m) => {
                world::get().borrow_mut().comp.mob.insert(entity, Mob::new(m));
                if m == mob::Player {
                    // Player-specific component stuffs.
                    world::get().borrow_mut().comp.map_memory.insert(
                        entity, MapMemory::new());
                }
            }
            todo => { println!("TODO: Handle spawn type {}", todo); unimplemented!(); }
        }

        entity.place(loc);

        entity
    }
}

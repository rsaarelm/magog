use location::{Location};
use {EntityKind, MobKind};
use mob::Mob;
use mob;
use world;
use entity::{Entity};
use map_memory::{MapMemory};

#[deriving(Clone)]
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
        let entity = world::with_mut(|w| {
            let entity = w.ecs.new_entity();

            w.comp.kind.insert(entity, self.kind);

            match self.kind {
                MobKind(m) => {
                    w.comp.mob.insert(entity, Mob::new(m));
                    if m == mob::Player {
                        // Player-specific component stuffs.
                        w.comp.map_memory.insert(
                            entity, MapMemory::new());
                    }
                }
                todo => { println!("TODO: Handle spawn type {}", todo); unimplemented!(); }
            };
            entity
        });

        entity.place(loc);

        entity
    }
}

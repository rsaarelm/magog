use location::{Location};
use mob::{Mob, MobType};
use world;
use EntityKind;
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

            w.kinds_mut().insert(entity, self.kind);

            match self.kind {
                EntityKind::Mob(m) => {
                    w.mobs_mut().insert(entity, Mob::new(m));
                    if m == MobType::Player {
                        // Player-specific component stuffs.
                        w.map_memories_mut().insert(
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

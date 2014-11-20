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

            w.comp.kind.insert(entity, self.kind);

            match self.kind {
                EntityKind::Mob(m) => {
                    w.comp.mob.insert(entity, Mob::new(m));
                    if m == MobType::Player {
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

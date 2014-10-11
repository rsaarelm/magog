use location::{Location};
use {EntityKind, MobKind};
use mob::Mob;
use world;
use entity::{Entity};

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
        entity.place(loc);

        world::get().borrow_mut().comp.kind.insert(entity, self.kind);

        match self.kind {
            MobKind(m) => world::get().borrow_mut().comp.mob.insert(entity, Mob::new(m)),
            todo => { println!("TODO: Handle spawn type {}", todo); unimplemented!(); }
        }

        entity
    }
}

use ecs::{Component};
use entity::{Entity};
use world;

#[derive(Copy)]
pub struct Prototype {
    pub target: Entity
}

impl Prototype {
    pub fn new(parent: Option<Entity>) -> Prototype {
        Prototype {
            target: world::with_mut(|w| w.ecs.new_entity(parent))
        }
    }
}

impl<C: Component> Fn(C,) -> Prototype for Prototype {
    extern "rust-call" fn call(&self, (comp,): (C,)) -> Prototype {
        comp.add_to(self.target);
        *self
    }
}

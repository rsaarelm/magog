use std::rc::Rc;
use std::cell::RefCell;
use entity::Entity;
use ecs::{Ecs, Component};
use {EntityKind};
use mob::Mob;
use desc::Desc;
use map_memory::MapMemory;

/// Generic components used by the game.
#[deriving(Encodable, Decodable)]
pub struct Comp {
    pub kind: Component<EntityKind>,
    pub mob: Component<Mob>,
    pub map_memory: Component<MapMemory>,
    pub desc: Component<Desc>,
}

impl Comp {
    pub fn new(ecs: Rc<RefCell<Ecs>>) -> Comp {
        Comp {
            kind: Component::new(ecs.clone()),
            mob: Component::new(ecs.clone()),
            map_memory: Component::new(ecs.clone()),
            desc: Component::new(ecs.clone()),
        }
    }

    /// Remove entity from all member components.
    pub fn remove(&mut self, e: Entity) {
        // LABYRINTH OF COMPONENTS
        // All Comp member components must be included here.
        self.kind.remove(e);
        self.mob.remove(e);
        self.map_memory.remove(e);
        self.desc.remove(e);
    }
}

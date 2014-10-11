use entity::{Entity};
use ecs::{Component};
use {EntityKind};
use mob::{Mob};

/// Generic components used by the game.
#[deriving(Encodable, Decodable)]
pub struct Comp {
    pub kind: Component<EntityKind>,
    pub mob: Component<Mob>,
}

impl Comp {
    pub fn new() -> Comp {
        Comp {
            kind: Component::new(),
            mob: Component::new(),
        }
    }

    /// Delete entity from all memeber components.
    pub fn delete(&mut self, e: Entity) {
        // All member components must be included here.
        self.kind.delete(e);
        self.mob.delete(e);
    }
}

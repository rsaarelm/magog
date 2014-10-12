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

    /// Remove entity from all memeber components.
    pub fn remove(&mut self, e: Entity) {
        // LABYRINTH OF COMPONENTS
        // All Comp member components must be included here.
        self.kind.remove(e);
        self.mob.remove(e);
    }
}

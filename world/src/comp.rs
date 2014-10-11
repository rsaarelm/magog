use serialize::{Decodable, Decoder, Encodable, Encoder};
use ecs::{Component, Entity};

/// Generic components used by the game.
#[deriving(Encodable, Decodable)]
pub struct Comp {
    pub sprite: Component<Sprite>,
    pub stats: Component<Stats>,
}

impl Comp {
    pub fn new() -> Comp {
        Comp {
            sprite: Component::new(),
            stats: Component::new(),
        }
    }

    /// Delete entity from all memeber components.
    pub fn delete(&mut self, e: Entity) {
        // All member components must be included here.
        self.sprite.delete(e);
        self.stats.delete(e);
    }
}

// XXX: Placeholder, get some actual components in, remove this.
#[deriving(Encodable, Decodable)]
pub struct Sprite {
    pub name: String,
    pub idx: uint,
}

// XXX: Placeholder, get some actual components in, remove this.
#[deriving(Encodable, Decodable)]
pub struct Stats {
    pub strength: int,
    pub dex: int,
}


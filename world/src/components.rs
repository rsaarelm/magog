use std::collections::HashSet;
use calx::Rgb;
use location::Location;
use {Biome};
use mob;

/// Map field-of-view and remembered terrain.
#[deriving(Clone, Show, Encodable, Decodable)]
pub struct MapMemory {
    pub seen: HashSet<Location>,
    pub remembered: HashSet<Location>,
}

impl MapMemory {
    pub fn new() -> MapMemory {
        MapMemory {
            seen: HashSet::new(),
            remembered: HashSet::new(),
        }
    }
}


/// Entity name and appearance.
#[deriving(Clone, Show, Encodable, Decodable)]
pub struct Desc {
    pub name: String,
    pub icon: uint,
    pub color: Rgb,
}

impl Desc {
    pub fn new(name: String, icon: uint, color: Rgb) -> Desc {
        Desc {
            name: name,
            icon: icon,
            color: color,
        }
    }
}

/// Spawning properties for prototype objects.
#[deriving(Copy, Clone, Show, Encodable, Decodable)]
pub struct Spawn {
    /// Types of areas where this entity can spawn.
    pub biome: Biome,
    /// Unlikeliness of the entity to spawn. Rarity is the inverse of an
    /// entity's weight in the spawning probability distribution. Entities
    /// with rarity zero do not spawn spontaneously.
    pub rarity: uint,
    /// Minimum depth where the entity will show up. More powerful entities
    /// only start showing up in large depths.
    pub min_depth: uint,
}

// TODO: Kind can be deprecated eventually I think. Just infer the entity type
// from components present that do actual stuff.

/// General type of a game entity.
#[deriving(Copy, Eq, PartialEq, Clone, Show, Encodable, Decodable)]
pub enum Kind {
    /// An active, mobile entity like the player or the NPCs.
    Mob(mob::MobType),
    /// An entity that can be picked up and used in some way.
    Item, // TODO ItemType data.
    /// A background item that doesn't do much.
    Prop,
    /// A static object that does things when stepped on.
    Node,
}

/// Unchanging statistics for mobs.
#[deriving(Copy, Clone, Show, Encodable, Decodable)]
pub struct MobStats {
    pub power: int,
    pub intrinsics: i32,
}

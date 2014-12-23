use std::collections::HashSet;
use calx::Rgb;
use location::Location;
use {Biome};
use mob;
use entity::Entity;
use world;

macro_rules! impl_component {
    { $comp:ty, $method:ident } => {
        impl Component for $comp {
            // XXX: Figure out how to move self into the closure to
            // get rid of the .clone.
            fn add_to(self, e: Entity) { world::with_mut(|w| w.$method().insert(e, self.clone())) }
        }
    }
}

pub trait Component {
    /// Create an uniform syntax for attaching components to entities to allow
    /// a fluent API for constructing prototypes.
    fn add_to(self, e: Entity);
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

impl_component!(Desc, descs_mut)

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
    /// Placeholder while phasing out Kind.
    Unknown,
}

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

impl_component!(MapMemory, map_memories_mut)

/// Unchanging statistics for mobs.
#[deriving(Copy, Clone, Show, Encodable, Decodable)]
pub struct MobStat {
    pub power: int,
    pub intrinsics: i32,
}

impl_component!(MobStat, mob_stats_mut)

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

impl_component!(Spawn, spawns_mut)

#[deriving(Copy)]
pub struct Prototype {
    target: Entity
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

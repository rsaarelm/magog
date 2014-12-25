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

    pub category: Category,
}

impl_component!(Spawn, spawns_mut)

#[deriving(Copy, Clone, Eq, PartialEq, Show, Encodable, Decodable)]
pub enum Category {
    Mob = 0b1,
    Item = 0b10,

    Anything = 0b11111111,
}

#[deriving(Copy, Clone, Show, Encodable, Decodable)]
pub struct Brain {
    pub state: BrainState,
    pub alignment: Alignment
}

impl_component!(Brain, brains_mut)

/// Mob behavior state.
#[deriving(Copy, Clone, Eq, PartialEq, Show, Encodable, Decodable)]
pub enum BrainState {
    /// AI mob is inactive, but can be startled into action by noise or
    /// motion.
    Asleep,
    /// AI mob is looking for a fight.
    Hunting,
    /// Mob is under player control.
    PlayerControl,
}

/// Used to determine who tries to fight whom.
#[deriving(Copy, Clone, Eq, PartialEq, Show, Encodable, Decodable)]
pub enum Alignment {
    /// Attack anything and everything.
    Chaotic,
    /// Player alignment. The noble path of slaughtering everything that moves
    /// and gets in the way of taking their shiny stuff.
    Good,
    /// Enemy alignment. The foul cause of working together to defend your
    /// home and belongings against a powerful and devious intruder.
    Evil,
}

/// Damage state component. The default state is undamaged and unarmored.
#[deriving(Copy, Clone, Show, Default, Encodable, Decodable)]
pub struct Health {
    /// The more wounds you have, the more hurt you are. How much damage you
    /// can take before dying depends on entity power level, not described by
    /// Wounds component. Probably in MobStat or something.
    pub wounds: uint,
    /// Armor points get eaten away before you start getting wounds.
    pub armor: uint,
}

impl_component!(Health, healths_mut)

////////////////////////////////////////////////////////////////////////

#[deriving(Copy)]
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

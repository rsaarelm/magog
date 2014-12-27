use std::collections::HashSet;
use calx::Rgb;
use location::Location;
use {Biome};
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
#[deriving(Clone, Show, RustcEncodable, RustcDecodable)]
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

impl_component!(Desc, descs_mut);

/// Map field-of-view and remembered terrain.
#[deriving(Clone, Show, RustcEncodable, RustcDecodable)]
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

impl_component!(MapMemory, map_memories_mut);

/// Unchanging statistics for mobs.
#[deriving(Copy, Clone, Show, RustcEncodable, RustcDecodable)]
pub struct MobStat {
    pub power: int,
    pub intrinsics: u32,
}

impl_component!(MobStat, mob_stats_mut);

#[deriving(Copy, Eq, PartialEq, Clone, Show, RustcEncodable, RustcDecodable)]
pub enum Intrinsic {
    /// Moves 1/3 slower than usual.
    Slow        = 0b1,
    /// Moves 1/3 faster than usual, stacks with Quick status.
    Fast        = 0b10,
    /// Can manipulate objects and doors.
    Hands       = 0b100,
}

/// Spawning properties for prototype objects.
#[deriving(Copy, Clone, Show, RustcEncodable, RustcDecodable)]
pub struct Spawn {
    /// Types of areas where this entity can spawn.
    pub biome: Biome,
    /// Weight of this entity in the random sampling distribution.
    pub commonness: uint,
    /// Minimum depth where the entity will show up. More powerful entities
    /// only start showing up in large depths.
    pub min_depth: uint,

    pub category: Category,
}

impl_component!(Spawn, spawns_mut);

#[deriving(Copy, Clone, Eq, PartialEq, Show, RustcEncodable, RustcDecodable)]
pub enum Category {
    Mob = 0b1,
    Item = 0b10,

    Anything = 0b11111111,
}

#[deriving(Copy, Clone, Show, RustcEncodable, RustcDecodable)]
pub struct Brain {
    pub state: BrainState,
    pub alignment: Alignment
}

impl_component!(Brain, brains_mut);

/// Mob behavior state.
#[deriving(Copy, Clone, Eq, PartialEq, Show, RustcEncodable, RustcDecodable)]
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
#[deriving(Copy, Clone, Eq, PartialEq, Show, RustcEncodable, RustcDecodable)]
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
#[deriving(Copy, Clone, Show, Default, RustcEncodable, RustcDecodable)]
pub struct Health {
    /// The more wounds you have, the more hurt you are. How much damage you
    /// can take before dying depends on entity power level, not described by
    /// Wounds component. Probably in MobStat or something.
    pub wounds: int,
    /// Armor points get eaten away before you start getting wounds.
    pub armor: int,
}

impl_component!(Health, healths_mut);

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

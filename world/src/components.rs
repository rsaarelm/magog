use std::collections::VecMap;
use std::collections::HashSet;
use util::Rgb;
use location::Location;
use {Biome};
use entity::Entity;
use item::{ItemType};
use ability::Ability;
use stats::Stats;
use component_ref::{ComponentRef, ComponentRefMut};
use world::{self, WorldState};

macro_rules! components {
    {
        // Declare the list of types which are included as components in the
        // game's entity component system. Also declare the non-mutable and
        // mutable accessor names for them. Example
        //
        // ```notrust
        //     [Mesh, meshes, meshes_mut],
        // ```
        $([$comp:ty, $access:ident, $access_mut:ident],)+
    } => {
        // The master container for all the components.
#[derive(RustcEncodable, RustcDecodable)]
        pub struct Comps {
            $($access: VecMap<$comp>,)+
        }

        /// Container for all regular entity components.
        impl Comps {
            pub fn new() -> Comps {
                Comps {
                    $($access: VecMap::new(),)+
                }
            }

            /// Remove the given entity from all the contained components.
            pub fn remove(&mut self, Entity(idx): Entity) {
                $(self.$access.remove(&idx);)+
            }
        }


        // Implement the Componet trait for the type, this provides an uniform
        // syntax for adding component values to entities used by the entity
        // factory.
        $(
            impl Component for $comp {
                // XXX: Figure out how to move self into the closure to
                // get rid of the .clone.
                fn add_to(self, e: Entity) { world::with_mut(|w| w.$access_mut().insert(e, self.clone())) }
            }
        )+


        // Implement the trait for accessing all the components that
        // WorldState will implement
        pub trait ComponentAccess<'a> {
            $(
            fn $access(&'a self) -> ComponentRef<'a, $comp>;
            fn $access_mut(&'a mut self) -> ComponentRefMut<'a, $comp>;
            )+
        }

        impl<'a> ComponentAccess<'a> for WorldState {
            $(
            fn $access(&'a self) -> ComponentRef<'a, $comp> {
                ComponentRef::new(&self.ecs, &self.comps.$access)
            }
            fn $access_mut(&'a mut self) -> ComponentRefMut<'a, $comp> {
                ComponentRefMut::new(&mut self.ecs, &mut self.comps.$access)
            }
            )+
        }
    }
}

pub trait Component {
    /// Create an uniform syntax for attaching components to entities to allow
    /// a fluent API for constructing prototypes.
    fn add_to(self, e: Entity);
}

////////////////////////////////////////////////////////////////////////

/// Entity name and appearance.
#[derive(Clone, Show, RustcEncodable, RustcDecodable)]
pub struct Desc {
    pub name: String,
    pub icon: usize,
    pub color: Rgb,
}

impl Desc {
    pub fn new(name: String, icon: usize, color: Rgb) -> Desc {
        Desc {
            name: name,
            icon: icon,
            color: color,
        }
    }
}


/// Map field-of-view and remembered terrain.
#[derive(Clone, Show, RustcEncodable, RustcDecodable)]
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


/// Spawning properties for prototype objects.
#[derive(Copy, Clone, Show, RustcEncodable, RustcDecodable)]
pub struct Spawn {
    /// Types of areas where this entity can spawn.
    pub biome: Biome,
    /// Weight of this entity in the random sampling distribution.
    pub commonness: u32,
    /// Minimum depth where the entity will show up. More powerful entities
    /// only start showing up in large depths.
    pub min_depth: i32,

    pub category: Category,
}

#[derive(Copy, Clone, Eq, PartialEq, Show, RustcEncodable, RustcDecodable)]
pub enum Category {
    Mob = 0b1,

    Consumable = 0b10,
    Equipment = 0b100,

    Item = 0b110,

    Anything = -1,
}


#[derive(Copy, Clone, Show, RustcEncodable, RustcDecodable)]
pub struct Brain {
    pub state: BrainState,
    pub alignment: Alignment
}

/// Mob behavior state.
#[derive(Copy, Clone, Eq, PartialEq, Show, RustcEncodable, RustcDecodable)]
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
#[derive(Copy, Clone, Eq, PartialEq, Show, RustcEncodable, RustcDecodable)]
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
#[derive(Copy, Clone, Show, Default, RustcEncodable, RustcDecodable)]
pub struct Health {
    /// The more wounds you have, the more hurt you are. How much damage you
    /// can take before dying depends on entity power level, not described by
    /// Wounds component. Probably in MobStat or something.
    pub wounds: i32,
    /// Armor points get eaten away before you start getting wounds.
    pub armor: i32,
}


/// Items can be picked up and carried and they do stuff.
#[derive(Clone, Show, RustcEncodable, RustcDecodable)]
pub struct Item {
    pub item_type: ItemType,
    pub ability: Ability,
}


/// Stats cache is a transient component made from adding up a mob's intrinsic
/// stats and the stat bonuses of its equipment and whatever spell effects may
/// apply.
pub type StatsCache = Option<Stats>;

////////////////////////////////////////////////////////////////////////

// Component loadout for the game.
components! {
    [Desc, descs, descs_mut],
    [MapMemory, map_memories, map_memories_mut],
    [Stats, stats, stats_mut],
    [Spawn, spawns, spawns_mut],
    [Health, healths, healths_mut],
    [Brain, brains, brains_mut],
    [Item, items, items_mut],
    [StatsCache, stats_caches, stats_caches_mut],
}

////////////////////////////////////////////////////////////////////////

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

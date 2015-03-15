use std::collections::HashSet;
use util::Rgb;
use location::Location;
use {Biome};
use item::{ItemType};
use ability::Ability;
use stats::Stats;

/// Dummy component to mark prototype objects
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct IsPrototype;

/// Entity name and appearance.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Desc {
    pub name: String,
    pub icon: usize,
    pub color: Rgb,
}

impl Desc {
    pub fn new(name: &str, icon: usize, color: Rgb) -> Desc {
        // XXX: Not idiomatic to set this to be called with a non-owned
        // &str instead of a String, I just want to get away from typing
        // .to_string() everywhere with the calls that mostly use string
        // literals.
        Desc {
            name: name.to_string(),
            icon: icon,
            color: color,
        }
    }
}


/// Map field-of-view and remembered terrain.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
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
#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
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

impl Spawn {
    pub fn new(category: Category) -> Spawn {
        Spawn {
            biome: Biome::Overland,
            commonness: 1000,
            min_depth: 1,
            category: category,
        }
    }

    /// Set the biome(s) where this entity can be spawned. By default entities
    /// can spawn anywhere.
    pub fn biome(mut self, biome: Biome) -> Spawn {
        self.biome = biome; self
    }

    /// Set the minimum depth where the entity can spawn. More powerful
    /// entities should only spawn in greater depths. By default this is 1.
    pub fn depth(mut self, min_depth: i32) -> Spawn {
        self.min_depth = min_depth; self
    }

    /// Set the probability for this entity to spawn. Twice as large is twice
    /// as common. The default is 1000.
    pub fn commonness(mut self, commonness: u32) -> Spawn {
        assert!(commonness > 0);
        self.commonness = commonness; self
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, RustcEncodable, RustcDecodable)]
pub enum Category {
    Mob = 0b1,

    Consumable = 0b10,
    Equipment = 0b100,

    Item = 0b110,

    Anything = -1,
}


#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Brain {
    pub state: BrainState,
    pub alignment: Alignment
}

/// Mob behavior state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, RustcEncodable, RustcDecodable)]
pub enum BrainState {
    /// AI mob is inactive, but can be startled into action by noise or
    /// motion.
    Asleep,
    /// AI mob is looking for a fight.
    Hunting,
    /// AI mob is wandering around.
    Roaming,
    /// Mob is under player control.
    PlayerControl,
}

/// Used to determine who tries to fight whom.
#[derive(Copy, Clone, Eq, PartialEq, Debug, RustcEncodable, RustcDecodable)]
pub enum Alignment {
    Berserk,
    Phage,
    Indigenous,
    Colonist,
}


/// Damage state component. The default state is undamaged and unarmored.
#[derive(Copy, Clone, Debug, Default, RustcEncodable, RustcDecodable)]
pub struct Health {
    /// The more wounds you have, the more hurt you are. How much damage you
    /// can take before dying depends on entity power level, not described by
    /// Wounds component. Probably in MobStat or something.
    pub wounds: i32,
    /// Armor points get eaten away before you start getting wounds.
    pub armor: i32,
}


/// Items can be picked up and carried and they do stuff.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Item {
    pub item_type: ItemType,
    pub ability: Ability,
}


/// Stats cache is a transient component made from adding up a mob's intrinsic
/// stats and the stat bonuses of its equipment and whatever spell effects may
/// apply.
pub type StatsCache = Option<Stats>;


/// Belong to a zone.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Colonist {
    pub home_base: String,
}

impl Colonist {
    // Bases will be assigned when the unit is deployed.
    pub fn new() -> Colonist { Colonist { home_base: String::new() } }
}

use std::collections::HashSet;
use util::Rgb;
use location::Location;
use {Biome};
use item::{ItemType};
use ability::Ability;
use stats::Stats;

/// Entity name and appearance.
#[derive(Clone, Show, RustcEncodable, RustcDecodable)]
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

impl Spawn {
    pub fn new(category: Category) -> Spawn {
        Spawn {
            biome: Biome::Anywhere,
            commonness: 1000,
            min_depth: 1,
            category: category,
        }
    }

    pub fn biome(mut self, biome: Biome) -> Spawn {
        self.biome = biome; self
    }

    pub fn depth(mut self, min_depth: i32) -> Spawn {
        self.min_depth = min_depth; self
    }

    pub fn commonness(mut self, commonness: u32) -> Spawn {
        self.commonness = commonness; self
    }
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

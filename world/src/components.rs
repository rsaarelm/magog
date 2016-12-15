use std::convert::Into;
use calx_color::Rgba;
use calx_resource::Resource;
use item::ItemType;
use ability::Ability;
use stats::Stats;
use location::Location;
use location_set::LocationSet;
use brush::Brush;
use FovStatus;

/// Entity name and appearance.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Desc {
    pub name: String,
    pub brush: Resource<Brush>,
}

impl Desc {
    pub fn new(name: &str, brush: &str) -> Desc {
        let brush = Resource::new(brush.to_string())
                        .expect(&format!("Brush '{}' not found", brush));
        // XXX: Not idiomatic to set this to be called with a non-owned
        // &str instead of a String, I just want to get away from typing
        // .to_string() everywhere with the calls that mostly use string
        // literals.
        Desc {
            name: name.to_string(),
            brush: brush,
        }
    }
}


/// Map field-of-view and remembered terrain.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct MapMemory {
    pub seen: LocationSet,
    pub remembered: LocationSet,
}

impl MapMemory {
    pub fn new() -> MapMemory {
        MapMemory {
            seen: LocationSet::new(),
            remembered: LocationSet::new(),
        }
    }

    pub fn status(&self, loc: Location) -> Option<FovStatus> {
        if self.seen.contains(&loc) {
            Some(FovStatus::Seen)
        } else if self.remembered.contains(&loc) {
            Some(FovStatus::Remembered)
        } else {
            None
        }
    }
}


#[derive(Copy, Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Brain {
    pub state: BrainState,
    pub alignment: Alignment,
}

impl Brain {
    /// Create default enemy brain.
    pub fn enemy() -> Brain {
        Brain {
            state: BrainState::Asleep,
            alignment: Alignment::Evil,
        }
    }

    /// Create default player brain.
    pub fn player() -> Brain {
        Brain {
            state: BrainState::PlayerControl,
            alignment: Alignment::Good,
        }
    }
}

/// Mob behavior state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, RustcEncodable, RustcDecodable)]
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
#[derive(Copy, Clone, Eq, PartialEq, Debug, RustcEncodable, RustcDecodable)]
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
#[derive(Copy, Clone, Debug, Default, RustcEncodable, RustcDecodable)]
pub struct Health {
    /// The more wounds you have, the more hurt you are. How much damage you
    /// can take before dying depends on entity power level, not described by
    /// Wounds component. Probably in MobStat or something.
    pub wounds: i32,
    /// Armor points get eaten away before you start getting wounds.
    pub armor: i32,
}

impl Health {
    pub fn new() -> Health { Default::default() }
}


/// Items can be picked up and carried and they do stuff.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct Item {
    pub item_type: ItemType,
    pub ability: Ability,
}


/// Composite stats are generated from adding up a mob's intrinsic base stats
/// and stat bonuses from equipment it is wearing and any other transient
/// effects. They need to be updated whenever the relevant state of the entity
/// changes.
#[derive(Clone, Debug, RustcEncodable, RustcDecodable)]
pub struct CompositeStats(pub Stats);

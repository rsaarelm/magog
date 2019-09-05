use serde_derive::{Deserialize, Serialize};
use std::default::Default;
use std::ops::Add;

/// Stats specifies static bonuses for an entity. Stats values can be added
/// together to build composites. The Default value for Stats must be an
/// algebraic zero element, adding it to any Stats value must leave that value
/// unchanged.
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct Stats {
    /// Generic power level
    pub power: i32,
    /// Attack bonus
    pub attack: i32,
    /// Defense bonus
    pub defense: i32,
    /// Damage reduction
    pub armor: i32,
    /// Mana pool / mana drain
    pub mana: i32,
    /// Ranged attack range. Zero means no ranged capability.
    pub ranged_range: u32,
    /// Ranged attack power
    pub ranged_power: i32,

    /// Bit flags for intrinsics
    pub intrinsics: u32,
}

impl Stats {
    pub fn new(power: i32, intrinsics: &[Intrinsic]) -> Stats {
        let intrinsics = intrinsics.iter().fold(0, |acc, &i| acc | (1 << i as u32));
        Stats {
            power,
            intrinsics,
            attack: power,
            ..Default::default()
        }
    }

    pub fn mana(self, mana: i32) -> Stats { Stats { mana, ..self } }
    pub fn armor(self, armor: i32) -> Stats { Stats { armor, ..self } }
    pub fn attack(self, attack: i32) -> Stats { Stats { attack, ..self } }
    pub fn defense(self, defense: i32) -> Stats { Stats { defense, ..self } }
    pub fn ranged_range(self, ranged_range: u32) -> Stats {
        Stats {
            ranged_range,
            ..self
        }
    }
    pub fn ranged_power(self, ranged_power: i32) -> Stats {
        Stats {
            ranged_power,
            ..self
        }
    }

    pub fn add_intrinsic(&mut self, intrinsic: Intrinsic) {
        self.intrinsics |= 1 << intrinsic as u32;
    }
}

impl Add<Stats> for Stats {
    type Output = Stats;
    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, other: Stats) -> Stats {
        Stats {
            power: self.power + other.power,
            attack: self.attack + other.attack,
            defense: self.defense + other.defense,
            armor: self.armor + other.armor,
            mana: self.mana + other.mana,
            // XXX: Must be careful to have exactly one "ranged weapon" item
            // in the mix. A mob with a natural ranged attack equipping a
            // ranged weapon should *not* have the ranges added together.
            // On the other hand a "sniper scope" trinket could be a +2 range
            // type dealie.
            ranged_range: self.ranged_range + other.ranged_range,
            ranged_power: self.ranged_power + other.ranged_power,
            intrinsics: self.intrinsics | other.intrinsics,
        }
    }
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
/// Permanent creature properties.
pub enum Intrinsic {
    /// Moves 1/3 slower than usual, stacks with Slowed status.
    Slow,
    /// Moves 1/3 faster than usual, stacks with Hasted status.
    Quick,
    /// Can manipulate objects and doors.
    Hands,
    /// Explodes on death
    Deathsplosion,
}

use crate::{effect::Damage, Ability, AnimState, ItemType, Slot, World};
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::default::Default;
use std::ops::Add;

/// Stats specifies static bonuses for an entity. Stats values can be added
/// together to build composites. The Default value for Stats must be an
/// algebraic zero element, adding it to any Stats value must leave that value
/// unchanged.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
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
        let intrinsics =
            intrinsics.iter().fold(0, |acc, &i| acc | (1 << i as u32));
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

/// Damage state component. The default state is undamaged and unarmored.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
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

#[derive(
    Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug, Serialize, Deserialize,
)]
/// Temporary creature properties
pub enum Status {
    /// Creature is acting erratically
    Confused,
    /// Is dead (not undead-dead, no-longer-subject-to-animate-things-logic-dead)
    Dead,
    /// Moves 1/3 slower than usual, stacks with Slow intrinsic.
    Slowed,
    /// Moves 1/3 faster than usual, stacks with Quick intrinsic.
    Hasted,
    /// Creature is delayed.
    ///
    /// This gets jumped up every time after the creature acted.
    Delayed,
}

pub type Statuses = BTreeMap<Status, u32>;

/// Stats component in the ECS that supports caching applied modifiers for efficiency.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct StatsComponent {
    /// Base stats that are intrinsic to this entity
    pub base: Stats,
    /// Modified stats derived from base and various effects that apply.
    ///
    /// Must be explicitly regenerated whenever an attached stats-affecting entity changes.
    pub actual: Stats,
}

impl StatsComponent {
    pub fn new(base: Stats) -> StatsComponent {
        StatsComponent { base, actual: base }
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
    /// Always roaming, can't go to sleep state
    Hyperactive,
}

impl World {
    /// Return maximum health of an entity.
    pub fn max_hp(&self, e: Entity) -> i32 { self.stats(e).power }

    /// Return current health of an entity.
    pub fn hp(&self, e: Entity) -> i32 {
        self.max_hp(e)
            - if self.ecs().health.contains(e) {
                self.ecs().health[e].wounds
            } else {
                0
            }
    }

    /// Return the (composite) stats for an entity.
    ///
    /// Will return the default value for the Stats type (additive identity in the stat algebra)
    /// for entities that have no stats component defined.
    pub fn stats(&self, e: Entity) -> Stats {
        self.ecs()
            .stats
            .get(e)
            .map(|s| s.actual)
            .unwrap_or_default()
    }

    /// Return the base stats of the entity. Does not include any added effects.
    ///
    /// You usually want to use the `stats` method instead of this one.
    pub fn base_stats(&self, e: Entity) -> Stats {
        self.ecs().stats.get(e).map(|s| s.base).unwrap_or_default()
    }

    /// Return whether the entity has a specific intrinsic property (eg. poison resistance).
    pub fn has_intrinsic(&self, e: Entity, intrinsic: Intrinsic) -> bool {
        self.stats(e).intrinsics & (1 << intrinsic as u32) != 0
    }

    /// Return whether the entity has a specific temporary status
    pub fn has_status(&self, e: Entity, status: Status) -> bool {
        self.ecs()
            .status
            .get(e)
            .map_or(false, |s| s.contains_key(&status))
    }

    pub fn has_ability(&self, e: Entity, ability: Ability) -> bool {
        self.list_abilities(e).into_iter().any(|x| x == ability)
    }

    pub fn list_abilities(&self, e: Entity) -> Vec<Ability> {
        // Check for item abilities.
        if let Some(item) = self.ecs().item.get(e) {
            match item.item_type {
                ItemType::UntargetedUsable(ability) => {
                    return vec![ability];
                }
                ItemType::TargetedUsable(ability) => {
                    return vec![ability];
                }
                ItemType::Instant(ability) => {
                    return vec![ability];
                }
                _ => {}
            }
        }

        // Entity has no abilites.
        Vec::new()
    }

    pub(crate) fn damage(
        &mut self,
        e: Entity,
        amount: i32,
        damage_type: Damage,
        source: Option<Entity>,
    ) {
        if let Some(attacker) = source {
            self.notify_attacked_by(e, attacker);
        }

        let max_hp = self.max_hp(e);

        let mut hurt = false;
        let mut kill = false;
        if let Some(health) = self.ecs_mut().health.get_mut(e) {
            if amount > 0 {
                hurt = true;
                health.wounds += amount;

                if health.wounds > max_hp {
                    kill = true;
                }
            }
        }

        // Animate damage
        if hurt {
            let anim_tick = self.get_anim_tick();
            if let Some(anim) = self.ecs_mut().anim.get_mut(e) {
                anim.anim_start = anim_tick;
                anim.state = AnimState::MobHurt;
            }
        }

        if kill {
            if let Some(loc) = self.location(e) {
                if self.player_sees(loc) {
                    // TODO: message templating
                    msg!(
                        self,
                        "[One] {}.",
                        match damage_type {
                            Damage::Physical => "die[s]",
                            Damage::Fire => "burn[s] to ash",
                            Damage::Electricity => "[is] electrocuted",
                        }
                    )
                    .subject(e)
                    .send();
                }
                self.spawn_fx(loc, AnimState::Gib);
            }
            self.kill_entity(e);
        }
    }

    /// Do a single step of natural regeneration for a creature.
    ///
    /// Return amount of health gained, or None if at full health.
    pub(crate) fn tick_regeneration(&mut self, e: Entity) -> Option<i32> {
        let max_hp = self.max_hp(e);
        let increase = (max_hp / 30).max(1);

        let health = self.ecs_mut().health.get_mut(e)?;
        if health.wounds > 0 {
            let increase = increase.min(health.wounds);
            health.wounds -= increase;
            Some(increase)
        } else {
            None
        }
    }

    pub(crate) fn gain_status(
        &mut self,
        e: Entity,
        status: Status,
        duration: u32,
    ) {
        if duration == 0 {
            return;
        }

        if let Some(statuses) = self.ecs_mut().status.get_mut(e) {
            if let Some(current_duration) = statuses.get(&status).cloned() {
                if duration > current_duration {
                    // Pump up the duration.
                    statuses.insert(status, duration);
                }
            } else {
                // TODO: Special stuff when status first goes into effect goes here
                statuses.insert(status, duration);
            }
        }
    }

    pub(crate) fn tick_statuses(&mut self, e: Entity) {
        if let Some(statuses) = self.ecs_mut().status.get_mut(e) {
            let mut remove = Vec::new();

            for (k, d) in statuses.iter_mut() {
                *d -= 1;
                if *d == 0 {
                    remove.push(*k);
                }
            }

            // TODO: Special stuff when status goes out of effect for dropped statuses.
            for k in remove.into_iter() {
                statuses.remove(&k);
            }
        }
    }

    /// Rebuild cached derived stats of an entity.
    ///
    /// Must be explicitly called any time either the entity's base stats or anything relating to
    /// attached stat-affecting entities like equipped items is changed.
    pub(crate) fn rebuild_stats(&mut self, e: Entity) {
        if !self.ecs().stats.contains(e) {
            return;
        }

        // Start with the entity's base stats.
        let mut stats = self.base_stats(e);

        // Add in stat modifiers from equipped items.
        for &slot in Slot::equipment_iter() {
            if let Some(item) = self.entity_equipped(e, slot) {
                stats = stats + self.stats(item);
            }
        }

        // Set the derived stats.
        self.ecs_mut().stats[e].actual = stats;
    }

    /// Consume one unit of nutrition
    ///
    /// Return false if the entity has an empty stomach.
    pub(crate) fn consume_nutrition(&mut self, _: Entity) -> bool {
        // TODO nutrition system
        true
    }
}

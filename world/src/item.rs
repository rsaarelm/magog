use crate::command::ActionOutcome;
use crate::effect::Ability;
use crate::mutate::Mutate;
use crate::world::World;
use crate::Query;
use calx_ecs::Entity;
use serde_derive::{Deserialize, Serialize};
use std::slice;

pub const BAG_CAPACITY: u32 = 50;

/// Inventory slots.
#[derive(Copy, Eq, PartialEq, Clone, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Slot {
    Bag(u32),
    Head,
    Ranged,
    RightHand,
    Body,
    LeftHand,
    Feet,
    Trinket1,
    Trinket2,
    Trinket3,
}

impl Slot {
    pub fn is_equipment_slot(self) -> bool {
        match self {
            Slot::Bag(_) => false,
            _ => true,
        }
    }

    pub fn accepts(self, equip_type: Option<EquipType>) -> bool {
        use self::Slot::*;
        match self {
            RightHand => equip_type == Some(EquipType::Melee),
            LeftHand => false, // TODO: Shields etc
            Ranged => equip_type == Some(EquipType::Ranged),
            Head => equip_type == Some(EquipType::Head),
            Body => equip_type == Some(EquipType::Body),
            Feet => equip_type == Some(EquipType::Feet),
            Trinket1 | Trinket2 | Trinket3 => equip_type == Some(EquipType::Trinket),
            Bag(_) => true,
        }
    }

    pub fn equipment_iter() -> slice::Iter<'static, Slot> {
        use self::Slot::*;
        static EQUIPPED: [Slot; 9] = [
            Head, Ranged, RightHand, Body, LeftHand, Feet, Trinket1, Trinket2, Trinket3,
        ];

        EQUIPPED.iter()
    }
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum ItemType {
    MeleeWeapon,
    RangedWeapon,
    Helmet,
    Armor,
    Boots,
    /// Passive effects when equipped
    Trinket,
    /// Trigges an untargeted ability.
    ///
    /// By convention these items are spent after single use.
    UntargetedUsable(Ability),
    TargetedUsable(Ability),
    /// Consumed instantly when stepped on.
    Instant(Ability),
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EquipType {
    Melee,
    Ranged,
    Head,
    Body,
    Feet,
    Trinket,
}

/// Items can be picked up and carried and they do stuff.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub item_type: ItemType,
    /// How many uses a wand or similar has left.
    pub charges: u32,
}

/// An entity that can become a stack of multiple copies.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub struct Stacking {
    pub count: u32,
}

impl Default for Stacking {
    fn default() -> Self { Stacking { count: 1 } }
}

impl World {
    pub fn entities_in_bag(&self, parent: Entity) -> Vec<(Slot, Entity)> {
        self.entities_in(parent)
            .into_iter()
            .filter(|(slot, _)| if let Slot::Bag(_) = slot { true } else { false })
            .collect()
    }

    pub(crate) fn entity_take(&mut self, e: Entity, item: Entity) -> ActionOutcome {
        // Only mobs can take items.
        if !self.is_mob(e) {
            return None;
        }

        if !self.is_item(item) {
            return None;
        }

        // Somehow trying to pick up something we're inside of. Pls don't break the universe.
        if self.entity_contains(item, e) {
            panic!("Trying to pick up an entity you are inside of. This shouldn't happen");
        }

        // Item might go into a stack, look for stacks.
        if self.is_stackable(item) {
            let bag = self.entities_in_bag(e);
            for (_, e) in &bag {
                // Even if we could stack with this, it's already full, ignore.
                if self.count(*e) == self.max_stack_size(*e) {
                    continue;
                }

                if self.can_stack_with(item, *e) {
                    let stack_size = self.count(*e) + self.count(item);
                    let max_size = self.max_stack_size(*e);

                    if stack_size <= max_size {
                        // Merge into an existing stack, delete incoming item.
                        self.ecs_mut().stacking[*e].count = stack_size;
                        self.kill_entity(item);
                        // Item was consumed, so we're done here.
                        return Some(true);
                    } else {
                        // Top up the stack, our item remains so we keep looking for a place or
                        // more items to merge it with.
                        let overflow = stack_size - max_size;
                        self.ecs_mut().stacking[*e].count = max_size;
                        self.ecs_mut().stacking[item].count = overflow;
                    }
                }
            }
        }

        if let Some(slot) = self.free_bag_slot(e) {
            self.equip_item(item, e, slot);
            if self.is_player(e) {
                msg!(self, "[One] pick[s] up [a thing].")
                    .subject(e)
                    .object(item)
                    .send();
            }

            self.end_turn(e);
            Some(true)
        } else {
            // No more inventory space
            None
        }
    }

    pub fn can_stack_with(&self, e: Entity, other: Entity) -> bool {
        if !self.is_empty(e) || !self.is_empty(other) {
            // Sanity check: Never stack things that contain things.
            // (Things that can ever contain other things shouldn't be given the Stackable
            // component to begin with.)
            return false;
        }

        if !self.is_alive(e) || !self.is_alive(other) {
            return false;
        }

        let mut e = self.extract(e).unwrap();
        let mut other = self.extract(other).unwrap();

        if e.loadout.stacking.is_none() || other.loadout.stacking.is_none() {
            // Both must support stacking.
            return false;
        }

        // Stack components can be non-equal and the entities can still stack, so clear them now.
        e.loadout.stacking = None;
        other.loadout.stacking = None;

        e == other
    }

    pub fn is_stackable(&self, e: Entity) -> bool { self.ecs().stacking.contains(e) }
}

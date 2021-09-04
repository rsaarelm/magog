//! Item and equipment logic

use crate::{msg, Ability, ActionOutcome, Location, World};
use calx::{hex_neighbors, CellVector, HexGeom};
use calx_ecs::Entity;
use euclid::vec2;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};
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
    pub fn is_item(&self, e: Entity) -> bool { self.ecs().item.contains(e) }

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
                msg!("[One] pick[s] up [a thing].";
                    self.subject(e), self.object(item));
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

    /// Return count on entity if it's a stack
    pub fn count(&self, e: Entity) -> u32 {
        if let Some(stacking) = self.ecs().stacking.get(e) {
            debug_assert!(stacking.count >= 1, "Invalid item stack size");
            stacking.count
        } else {
            1
        }
    }

    pub fn max_stack_size(&self, e: Entity) -> u32 {
        if self.ecs().stacking.contains(e) {
            99
        } else {
            1
        }
    }

    /// Find a drop position for an item, trying to keep one item per cell.
    ///
    /// Dropping several items in the same location will cause them to spread out to the adjacent
    /// cells. If there is no room for the items to spread out, they will be stacked on the initial
    /// drop site.
    pub fn empty_item_drop_location(&self, origin: Location) -> Location {
        static MAX_SPREAD_DISTANCE: i32 = 8;
        let is_valid = |v: CellVector| {
            self.can_drop_item_at(origin.jump(self, v)) && v.hex_dist() <= MAX_SPREAD_DISTANCE
        };
        let mut seen = HashSet::new();
        let mut incoming = VecDeque::new();
        incoming.push_back(vec2(0, 0));

        while let Some(offset) = incoming.pop_front() {
            if seen.contains(&offset) {
                continue;
            }

            seen.insert(offset);

            let loc = origin.jump(self, offset);
            if self.item_at(loc).is_none() {
                return loc;
            }

            let current_dist = offset.hex_dist();
            for v in hex_neighbors(offset) {
                if v.hex_dist() > current_dist && !seen.contains(&v) && is_valid(v) {
                    incoming.push_back(v);
                }
            }
        }

        origin
    }

    /// Return first item at given location.
    pub fn item_at(&self, loc: Location) -> Option<Entity> {
        self.entities_at(loc).into_iter().find(|&e| self.is_item(e))
    }

    pub fn can_drop_item_at(&self, loc: Location) -> bool {
        if !self.is_valid_location(loc) {
            return false;
        }
        if self.terrain(loc).blocks_walk() {
            return false;
        }
        if self.terrain(loc).is_door() {
            return false;
        }
        true
    }

    pub fn item_type(&self, e: Entity) -> Option<ItemType> {
        self.ecs().item.get(e).and_then(|item| Some(item.item_type))
    }

    pub fn free_bag_slot(&self, e: Entity) -> Option<Slot> {
        (0..BAG_CAPACITY)
            .find(|&i| self.entity_equipped(e, Slot::Bag(i)).is_none())
            .map(Slot::Bag)
    }

    pub fn free_equip_slot(&self, e: Entity, item: Entity) -> Option<Slot> {
        Slot::equipment_iter()
            .find(|&&x| x.accepts(self.equip_type(item)) && self.entity_equipped(e, x).is_none())
            .cloned()
    }

    /// Return number of times item can be used.
    pub fn uses_left(&self, item: Entity) -> u32 {
        self.ecs().item.get(item).map_or(0, |i| i.charges)
    }

    pub fn destroy_after_use(&self, item: Entity) -> bool {
        // XXX: Fragile. What we want here is to tag potions and scrolls as destroyed when used and
        // wands to stick around. Current item data doesn't have is_potion or is_scroll, but
        // coincidentally the scrolls tend to be untargeted and the wands tend to be targeted
        // spells, so we'll just use that as proxy.
        self.ecs().item.get(item).map_or(false, |i| {
            if let ItemType::UntargetedUsable(_) = i.item_type {
                true
            } else {
                false
            }
        })
    }

    pub fn equip_type(&self, item: Entity) -> Option<EquipType> {
        use crate::ItemType::*;
        match self.item_type(item) {
            Some(MeleeWeapon) => Some(EquipType::Melee),
            Some(RangedWeapon) => Some(EquipType::Ranged),
            Some(Helmet) => Some(EquipType::Head),
            Some(Armor) => Some(EquipType::Body),
            Some(Boots) => Some(EquipType::Feet),
            Some(Trinket) => Some(EquipType::Trinket),
            _ => None,
        }
    }

    pub(crate) fn drain_charge(&mut self, item: Entity) {
        if self.destroy_after_use(item) {
            self.kill_entity(item);
        }

        if let Some(i) = self.ecs_mut().item.get_mut(item) {
            if i.charges > 0 {
                i.charges -= 1;
            }
        }
    }
}

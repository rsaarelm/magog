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
    UntargetedUsable(MagicEffect),
    TargetedUsable(MagicEffect),
    /// Consumed instantly when stepped on.
    Instant(MagicEffect),
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum MagicEffect {
    Heal,
    Confuse,
    Lightning,
    Fireball,
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

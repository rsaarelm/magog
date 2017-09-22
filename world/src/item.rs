use std::slice;

/// Inventory slots.
#[derive(Copy, Eq, PartialEq, Clone, Debug, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Slot {
    Spell1,
    Spell2,
    Spell3,
    Spell4,
    Spell5,
    Spell6,
    Spell7,
    Spell8,
    Melee,
    Ranged,
    Head,
    Body,
    Feet,
    TrinketF,
    TrinketG,
    TrinketH,
    TrinketI,
    InventoryJ,
    InventoryK,
    InventoryL,
    InventoryM,
    InventoryN,
    InventoryO,
    InventoryP,
    InventoryQ,
    InventoryR,
    InventoryS,
    InventoryT,
    InventoryU,
    InventoryV,
    InventoryW,
    InventoryX,
    InventoryY,
    InventoryZ,
}

impl Slot {
    pub fn is_equipment_slot(self) -> bool { (self as u32) <= (Slot::TrinketI as u32) }

    pub fn equipped_iter() -> slice::Iter<'static, Slot> {
        use Slot::*;
        static EQUIPPED: [Slot; 17] = [
            Spell1,
            Spell2,
            Spell3,
            Spell4,
            Spell5,
            Spell6,
            Spell7,
            Spell8,
            Melee,
            Ranged,
            Head,
            Body,
            Feet,
            TrinketF,
            TrinketG,
            TrinketH,
            TrinketI,
        ];

        EQUIPPED.iter()
    }

    pub fn iter() -> slice::Iter<'static, Slot> {
        use Slot::*;
        static ALL: [Slot; 34] = [
            Spell1,
            Spell2,
            Spell3,
            Spell4,
            Spell5,
            Spell6,
            Spell7,
            Spell8,
            Melee,
            Ranged,
            Head,
            Body,
            Feet,
            TrinketF,
            TrinketG,
            TrinketH,
            TrinketI,
            InventoryJ,
            InventoryK,
            InventoryL,
            InventoryM,
            InventoryN,
            InventoryO,
            InventoryP,
            InventoryQ,
            InventoryR,
            InventoryS,
            InventoryT,
            InventoryU,
            InventoryV,
            InventoryW,
            InventoryX,
            InventoryY,
            InventoryZ,
        ];

        ALL.iter()
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
    Spell,
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

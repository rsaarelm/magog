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

    pub fn accepts(self, equip_type: EquipType) -> bool {
        use Slot::*;
        match self {
            Spell1 => equip_type == EquipType::Spell,
            Spell2 => equip_type == EquipType::Spell,
            Spell3 => equip_type == EquipType::Spell,
            Spell4 => equip_type == EquipType::Spell,
            Spell5 => equip_type == EquipType::Spell,
            Spell6 => equip_type == EquipType::Spell,
            Spell7 => equip_type == EquipType::Spell,
            Spell8 => equip_type == EquipType::Spell,
            Melee => equip_type == EquipType::Melee,
            Ranged => equip_type == EquipType::Ranged,
            Head => equip_type == EquipType::Head,
            Body => equip_type == EquipType::Body,
            Feet => equip_type == EquipType::Feet,
            TrinketF => equip_type == EquipType::Trinket,
            TrinketG => equip_type == EquipType::Trinket,
            TrinketH => equip_type == EquipType::Trinket,
            TrinketI => equip_type == EquipType::Trinket,
            _ => false,
        }
    }

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

#[derive(Copy, Eq, PartialEq, Clone, Debug, Serialize, Deserialize)]
pub enum EquipType {
    Melee,
    Ranged,
    Head,
    Body,
    Feet,
    Spell,
    Trinket,
}

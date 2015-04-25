/// Inventory slots.
#[derive(Copy, Eq, PartialEq, Clone, Debug, PartialOrd, Ord, RustcEncodable, RustcDecodable)]
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
    pub fn is_gear_slot(self) -> bool {
        (self as u32) <= (Slot::TrinketI as u32)
    }
    pub fn is_bag_slot(self) -> bool {
        (self as u32) >= (Slot::InventoryJ as u32)
    }
}

#[derive(Copy, Eq, PartialEq, Clone, Debug, RustcEncodable, RustcDecodable)]
pub enum ItemType {
    MeleeWeapon,
    RangedWeapon,
    Helmet,
    Armor,
    Boots,
    Trinket,
    Spell,
    /// Can be carried and used later.
    Consumable,
    /// Consumed instantly when stepped on.
    Instant,
}

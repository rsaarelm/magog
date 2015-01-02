/// Inventory slots.
#[deriving(Copy, Eq, PartialEq, Clone, Show, FromPrimitive, PartialOrd, Ord, RustcEncodable, RustcDecodable)]
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
        (self as uint) <= (Slot::TrinketI as uint)
    }
    pub fn is_bag_slot(self) -> bool {
        (self as uint) >= (Slot::InventoryJ as uint)
    }
}

#[deriving(Copy, Eq, PartialEq, Clone, Show, RustcEncodable, RustcDecodable)]
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

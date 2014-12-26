/// Inventory slots.
#[deriving(Copy, Eq, PartialEq, Clone, Show, FromPrimitive, RustcEncodable, RustcDecodable)]
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
    Torso,
    Boots,
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

#[deriving(Copy, Eq, PartialEq, Clone, Show, RustcEncodable, RustcDecodable)]
pub enum ItemType {
    MeleeWeapon,
    RangedWeapon,
    /// Can be carried and used later.
    Consumable,
    /// Consumed instantly when stepped on.
    Instant,
}

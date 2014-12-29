/// Ability describes some way of affecting the game world. It is generally
/// attached to a mob or an item.
#[deriving(Copy, Clone, Show, RustcEncodable, RustcDecodable)]
pub enum Ability {
    /// Damage a target for a given amount.
    Damage(int),
    /// Heal a target for a given amount.
    Heal(int),
}

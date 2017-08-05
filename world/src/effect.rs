/// Game system effects on entities.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Effect {
    /// Heal an amount of damage.
    Heal(u32),
    /// Deal an amount of damage of a specific type.
    Hit { amount: u32, damage: Damage },
    /// Cause erratic behavior for a time.
    Confuse,
    /// Target mob learns current surroundings.
    ///
    /// Probably only does anything for player.
    MagicMap,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Damage {
    Physical,
    Fire,
    Electricity,
    Cold,
}

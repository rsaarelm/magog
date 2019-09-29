/// Game system effects on entities.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Effect {
    /// Deal an amount of damage of a specific type.
    Hit { amount: u32, damage: Damage },
    /// Cause erratic behavior for a time.
    Confuse,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum Damage {
    Physical,
    Fire,
    Electricity,
}

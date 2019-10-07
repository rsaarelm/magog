use serde_derive::{Deserialize, Serialize};

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

/// Actions a being can do
#[derive(Copy, Clone, Eq, PartialEq, Debug, Serialize, Deserialize)]
pub enum Ability {
    // --- Untargeted ---
    LightningBolt,
    // MagicMap

    // --- Targeted ---
    Fireball,
    Confuse,
}

impl Ability {
    pub fn is_targeted(self) -> bool {
        use Ability::*;
        match self {
            LightningBolt => false,
            _ => true,
        }
    }
}

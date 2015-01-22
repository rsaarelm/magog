use std::ops::{Add};

/// Stats specifies static bonuses for an entity. Stats values can be added
/// together to build composites. The Default value for Stats must be an
/// algebraic zero element, adding it to any Stats value must leave that value
/// unchanged.
#[derive(Copy, Clone, Show, Default, RustcEncodable, RustcDecodable)]
pub struct Stats {
    /// Generic power level
    pub power: i32,
    /// Bit flags for intrinsics
    pub intrinsics: u32,
}

impl Stats {
    pub fn new(power: i32, intrinsics: &[Intrinsic]) -> Stats {
        let mut intr = 0u32;
        for &i in intrinsics.iter() { intr = intr | (i as u32); }
        Stats {
            power: power,
            intrinsics: intr,
        }
    }
}

impl Add<Stats> for Stats {
    type Output = Stats;
    fn add(self, other: Stats) -> Stats {
        Stats {
            power: self.power + other.power,
            intrinsics: self.intrinsics | other.intrinsics,
        }
    }
}

#[derive(Copy, Eq, PartialEq, Clone, Show, RustcEncodable, RustcDecodable)]
pub enum Intrinsic {
    /// Moves 1/3 slower than usual.
    Slow        = 0b1,
    /// Moves 1/3 faster than usual, stacks with Quick status.
    Fast        = 0b10,
    /// Can manipulate objects and doors.
    Hands       = 0b100,
}

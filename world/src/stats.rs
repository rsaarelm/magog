/// Stats specifies static bonuses for an entity. Stats values can be added
/// together to build composites. The Default value for Stats must be an
/// algebraic zero element, adding it to any Stats value must leave that value
/// unchanged.
#[deriving(Copy, Clone, Show, Default, RustcEncodable, RustcDecodable)]
pub struct Stats {
    /// Generic power level
    pub power: i32,
    /// Bit flags for intrinsics
    pub intrinsics: u32,
}

impl Add<Stats, Stats> for Stats {
    fn add(self, other: Stats) -> Stats {
        Stats {
            power: self.power + other.power,
            intrinsics: self.intrinsics | other.intrinsics,
        }
    }
}

#[deriving(Copy, Eq, PartialEq, Clone, Show, RustcEncodable, RustcDecodable)]
pub enum Intrinsic {
    /// Moves 1/3 slower than usual.
    Slow        = 0b1,
    /// Moves 1/3 faster than usual, stacks with Quick status.
    Fast        = 0b10,
    /// Can manipulate objects and doors.
    Hands       = 0b100,
}

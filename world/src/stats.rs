/// Stats specifies static bonuses for an entity. Stats values can be added
/// together to build composites.
#[deriving(Copy, Clone, Show, Default, RustcEncodable, RustcDecodable)]
pub struct Stats {
    power: int,
}

impl Add<Stats, Stats> for Stats {
    fn add(self, other: Stats) -> Stats {
        Stats {
            power: self.power + other.power,
        }
    }
}

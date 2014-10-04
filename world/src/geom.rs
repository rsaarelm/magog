use std::cmp::max;
use std::f32::consts::PI;
use std::num::{signum, abs};
use calx::V2;

/// Unambiguous location in the game world.
#[deriving(Eq, PartialEq, Clone, Hash, Show)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    // TODO: Add third dimension for multiple persistent levels.
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location { Location { x: x, y: y } }

    pub fn to_v2(&self) -> V2<int> { V2(self.x as int, self.y as int) }

    /// Hex distance from another location.
    pub fn dist(&self, other: Location) -> int {
        // TODO: Does this handle edge wraparound with i8s correctly?
        let xd = (other.x - self.x) as int;
        let yd = (other.y - self.y) as int;
        if signum(xd) == signum(yd) {
            max(abs(xd), abs(yd))
        } else {
            abs(xd) + abs(yd)
        }
    }

    /// Return the hex direction that's closest to pointoing towards the given point.
    pub fn dir6_towards(&self, other: Location) -> V2<int> {
        return DIR6[
        match hexadecant(&V2((other.x - self.x) as int, (other.y - self.y) as int)) {
            14 | 15 => 0,
            0 | 1 | 2 | 3 => 1,
            4 | 5 => 2,
            6 | 7 => 3,
            8 | 9 | 10 | 11 => 4,
            12 | 13 => 5,
            _ => fail!("Bad hexadecant")
        }
        ];

        fn hexadecant(vec: &V2<int>) -> int {
            let width = PI / 8.0;
            let mut radian = (vec.0 as f32).atan2(-vec.1 as f32);
            if radian < 0.0 { radian += 2.0 * PI }
            return (radian / width).floor() as int;
        }
    }
}


impl Add<V2<int>, Location> for Location {
    fn add(&self, other: &V2<int>) -> Location {
        Location::new(
            (self.x as int + other.0) as i8,
            (self.y as int + other.1) as i8)
    }
}

impl Sub<Location, V2<int>> for Location {
    fn sub(&self, other: &Location) -> V2<int> {
        V2((self.x - other.x) as int, (self.y - other.y) as int)
    }
}

/// The six directions of a hex grid in the default order.
pub static DIR6: [V2<int>, ..6] = [
    V2(-1, -1),
    V2( 0, -1),
    V2( 1,  0),
    V2( 1,  1),
    V2( 0,  1),
    V2(-1,  0),
];

/// The eight directions of a rectangle grid in the default order.
pub static DIR8: [V2<int>, ..8] = [
    V2(-1, -1),
    V2( 0, -1),
    V2( 1, -1),
    V2( 1,  0),
    V2( 1,  1),
    V2( 0,  1),
    V2(-1,  1),
    V2(-1,  0),
];

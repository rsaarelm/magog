use std::cmp::max;
use std::f32::consts::PI;
use std::num::{signum, abs};
use calx::V2;

pub trait HexGeom {
    /// Hex distance represented by a vector.
    fn hex_dist(&self) -> int;

    /// The hexagonal base direction closest to this vector.
    fn dir6_towards(&self) -> V2<int>;
}

impl HexGeom for V2<int> {
    fn hex_dist(&self) -> int {
        let xd = self.0;
        let yd = self.1;
        if signum(xd) == signum(yd) {
            max(abs(xd), abs(yd))
        } else {
            abs(xd) + abs(yd)
        }
    }

    fn dir6_towards(&self) -> V2<int> {
        let hexadecant = {
            let width = PI / 8.0;
            let mut radian = (self.0 as f32).atan2(-self.1 as f32);
            if radian < 0.0 { radian += 2.0 * PI }
            (radian / width).floor() as int
        };

        DIR6[match hexadecant {
            14 | 15 => 0,
            0 | 1 | 2 | 3 => 1,
            4 | 5 => 2,
            6 | 7 => 3,
            8 | 9 | 10 | 11 => 4,
            12 | 13 => 5,
            _ => fail!("Bad hexadecant")
        }]
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

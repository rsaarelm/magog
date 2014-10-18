use std::cmp::max;
use std::num::{signum, abs};
use calx::V2;

/// Hex grid geometry for vectors.
pub trait HexGeom {
    /// Hex distance represented by a vector.
    fn hex_dist(&self) -> int;
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
}

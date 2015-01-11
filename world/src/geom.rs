use std::cmp::max;
use std::num::{SignedInt};
use util::V2;

/// Hex grid geometry for vectors.
pub trait HexGeom {
    /// Hex distance represented by a vector.
    fn hex_dist(&self) -> int;
}

impl HexGeom for V2<int> {
    fn hex_dist(&self) -> int {
        let xd = self.0;
        let yd = self.1;
        if xd.signum() == yd.signum() {
            max(xd.abs(), yd.abs())
        } else {
            xd.abs() + yd.abs()
        }
    }
}

use std::cmp::max;
use std::num::{SignedInt};
use calx::V2;

/// Hex grid geometry for vectors.
pub trait HexGeom {
    /// Hex distance represented by a vector.
    fn hex_dist(&self) -> i32;
}

impl HexGeom for V2<i32> {
    fn hex_dist(&self) -> i32 {
        let xd = self.0;
        let yd = self.1;
        if xd.signum() == yd.signum() {
            max(xd.abs(), yd.abs())
        } else {
            xd.abs() + yd.abs()
        }
    }
}

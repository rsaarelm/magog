use crate::cell::{Fov, PolarPoint};
use crate::hex::Dir6;
use crate::CellVector;
use euclid::vec2;

pub type HexFov<T> = Fov<HexPolarPoint, T>;

/// Points on a hex circle expressed in polar coordinates.
#[derive(Copy, Clone, PartialEq)]
pub struct HexPolarPoint {
    pos: f32,
    radius: u32,
}

impl HexPolarPoint {
    /// Index of the discrete hex cell along the circle that corresponds to this point.
    fn winding_index(self) -> i32 { (self.pos + 0.5).floor() as i32 }

    fn end_index(self) -> i32 { (self.pos + 0.5).ceil() as i32 }
}

impl PolarPoint for HexPolarPoint {
    fn unit_circle_endpoints() -> (Self, Self) {
        (
            HexPolarPoint {
                pos: 0.0,
                radius: 1,
            },
            HexPolarPoint {
                pos: 6.0,
                radius: 1,
            },
        )
    }

    fn is_below(&self, other: &HexPolarPoint) -> bool { self.winding_index() < other.end_index() }

    fn to_v2(&self) -> CellVector {
        if self.radius == 0 {
            return vec2(0, 0);
        }
        let index = self.winding_index();
        let sector = index.rem_euclid(self.radius as i32 * 6) / self.radius as i32;
        let offset = index.rem_euclid(self.radius as i32);

        let rod = Dir6::from_int(sector).to_v2();
        let tangent = Dir6::from_int(sector + 2).to_v2();

        rod * (self.radius as i32) + tangent * offset
    }

    fn expand(&self) -> Self {
        HexPolarPoint {
            pos: self.pos * (self.radius + 1) as f32 / self.radius as f32,
            radius: self.radius + 1,
        }
    }

    fn advance(&mut self) { self.pos = (self.pos + 0.5).floor() + 0.5; }
}

/// Special operations for FOV iterators using hex geometry.
pub trait HexFovIter: Sized {
    type Value;

    /// Extend a FOV iteration to make acute corners of fake-isometric hex map rooms visible.
    ///
    /// When using the standard hex logic, the two fake-isometric wall cells before the corner cell
    /// would block sight completely and make the corner invisible. This looks visually wrong, and
    /// can be fixed by applying this filter. The function parameter is used to check if a cell
    /// contains a wall. If there are three wall points that make up an acute corner, walking both
    /// of the closer two will make the third one added to the iterator output.
    ///
    /// This assumes you're using `HexPolarPoint` as the FOV type, in particular that the
    /// point moves clockwise as the coordinate value `advance`s.
    fn add_fake_isometric_acute_corners<F>(
        self,
        is_wall: F,
    ) -> AddFakeIsometricCorners<Self::Value, F, Self>
    where
        F: Fn(CellVector, &Self::Value) -> bool,
    {
        AddFakeIsometricCorners {
            inner: self,
            is_wall,
            prev: None,
            extra: None,
        }
    }
}

impl<T, I: Iterator<Item = (CellVector, T)>> HexFovIter for I {
    type Value = T;
}

pub struct AddFakeIsometricCorners<T, F, I> {
    inner: I,
    is_wall: F,
    prev: Option<(CellVector, T)>,
    extra: Option<(CellVector, T)>,
}

impl<T, F, I> Iterator for AddFakeIsometricCorners<T, F, I>
where
    T: Clone,
    F: Fn(CellVector, &T) -> bool,
    I: Iterator<Item = (CellVector, T)>,
{
    type Item = (CellVector, T);

    fn next(&mut self) -> Option<Self::Item> {
        use std::mem;

        if self.extra.is_some() {
            let mut ret = None;
            mem::swap(&mut ret, &mut self.extra);
            return ret;
        }

        let next = self.inner.next();

        if let (&Some(ref prev), &Some(ref next)) = (&self.prev, &next) {
            let prev_p = prev.0;
            let next_p = next.0;

            // Assume the polar coordinate rotates clockwise.
            if let Some(corner_p) = if next_p - prev_p == vec2(1, 1) {
                // Northeast side
                Some(prev_p + vec2(1, 0))
            } else if next_p - prev_p == vec2(-1, -1) {
                // Southwest side
                Some(prev_p + vec2(-1, 0))
            } else {
                None
            } {
                // We don't have the FOV value for the corner point, so just reuse the one from
                // `next` and hope it works out okay.
                if (self.is_wall)(prev.0, &prev.1)
                    && (self.is_wall)(next.0, &next.1)
                    && (self.is_wall)(corner_p, &next.1)
                {
                    // When the wall corner is found, push it to the extra slot to be returned
                    // in between this and the real next cell.
                    self.extra = Some((corner_p, next.1.clone()));
                }
            }
        }

        self.prev = next.clone();

        next
    }
}

#[cfg(test)]
mod test {
    use super::CellVector;
    use super::{HexFov, HexFovIter};
    use crate::{cell::FovValue, hex::HexGeom};
    use euclid::vec2;
    use std::collections::HashMap;
    use std::iter::FromIterator;

    #[derive(PartialEq, Eq, Clone)]
    struct Cell {
        range: i32,
    }

    impl FovValue for Cell {
        fn advance(&self, offset: CellVector) -> Option<Self> {
            if offset.hex_dist() < self.range {
                Some(self.clone())
            } else {
                None
            }
        }
    }

    #[test]
    fn trivial_fov() {
        // Just draw a small circle.
        let field: HashMap<CellVector, Cell> = HashMap::from_iter(HexFov::new(Cell { range: 2 }));
        assert!(field.contains_key(&vec2(1, 0)));
        assert!(!field.contains_key(&vec2(1, -1)));

        // Now test out the fake-isometric corners.
        let field: HashMap<CellVector, Cell> = HashMap::from_iter(
            HexFov::new(Cell { range: 2 }).add_fake_isometric_acute_corners(|_p, _t| true),
        );
        assert!(field.contains_key(&vec2(1, 0)));
        assert!(field.contains_key(&vec2(1, -1)));
    }
}

use num::Integer;
use euclid::Point2D;
use hex::Dir6;

pub trait FovValue: Eq + Clone {
    /// Construct a new FovValue based on previous one and a new point in the fov.
    fn advance(&self, offset: Point2D<i32>) -> Option<Self>;

    /// Optional method for showing acute corners of fake isometric rooms in fov.
    fn is_fake_isometric_wall(&self, offset: Point2D<i32>) -> bool {
        let _ = offset;
        false
    }
}

/// Field of view iterator for a hexagonal map.
///
/// Takes a function that maps cells into user-specified values and indicates when traversal should
/// stop.
pub struct HexFov<T> {
    stack: Vec<Sector<T>>,
    /// Extra values generated by special cases.
    side_channel: Vec<(Point2D<i32>, T)>,
}

impl<T: FovValue> HexFov<T> {
    pub fn new(init: T) -> HexFov<T> {
        // We could run f for (0, 0) here, but the traditional way for the FOV to work is to only
        // consider your surroundings, not the origin site itself.
        HexFov {
            stack: vec![Sector {
                            begin: PolarPoint::new(0.0, 1),
                            pt: PolarPoint::new(0.0, 1),
                            end: PolarPoint::new(6.0, 1),
                            prev_value: init.clone(),
                            group_value: None,
                        }],
            // The FOV algorithm will not generate the origin point, so we use
            // the side channel to explicitly add it in the beginning.
            side_channel: vec![(Point2D::new(0, 0), init)],
        }
    }
}

impl<T: FovValue> Iterator for HexFov<T> {
    type Item = (Point2D<i32>, T);
    fn next(&mut self) -> Option<(Point2D<i32>, T)> {
        if let Some(ret) = self.side_channel.pop() {
            return Some(ret);
        }

        if let Some(mut current) = self.stack.pop() {
            let current_value = current.prev_value.advance(current.pt.to_v2());

            if current.pt.is_below(current.end) && current_value.is_some() {
                let pos = current.pt.to_v2();
                let current_value = current_value.unwrap();

                match current.group_value {
                    None => {
                        // Beginning of group, value isn't set.
                        current.group_value = Some(current_value.clone());
                    }
                    Some(ref g) if g.clone() != current_value => {
                        // Value changed, branch out.

                        // Add the rest of this sector with the new value.
                        self.stack.push(Sector {
                            begin: current.pt,
                            pt: current.pt,
                            end: current.end,
                            prev_value: current.prev_value,
                            group_value: Some(current_value.clone()),
                        });

                        // Branch further on the arc processed so far.
                        self.stack.push(Sector {
                            begin: current.begin.further(),
                            pt: current.begin.further(),
                            end: current.pt.further(),
                            prev_value: g.clone(),
                            group_value: None,
                        });
                        return self.next();
                    }
                    _ => {}
                }

                // Current value and group value are assumed to be the same from here on.
                assert!(current.group_value == Some(current_value.clone()));

                // Hack for making acute corner tiles of fake-isometric rooms visible.

                // We're moving along a vertical line on the hex circle, so there are side
                // points to check.
                if let Some(side_pos) = current.pt.side_point() {

                    let next = current.pt.next();
                    // If the next cell is within the current span and the current cell is
                    // wallform,
                    if next.is_below(current.end) &&
                       current.prev_value.is_fake_isometric_wall(current.pt.to_v2()) {
                        // and if the next cell is visible,
                        if let Some(next_value) = current.prev_value.advance(next.to_v2()) {
                            // and if the current and the next cell are in the same value group,
                            // both the next cell and the third corner point cell are
                            // wallforms, and the side point would not be otherwise
                            // visible:
                            if next_value == current.prev_value &&
                               next_value.is_fake_isometric_wall(next.to_v2()) &&
                               current.prev_value.advance(side_pos).is_none() &&
                               current.prev_value.is_fake_isometric_wall(side_pos) {
                                // Add the side point to the side channel.
                                self.side_channel.push((side_pos, current.prev_value.clone()));
                            }
                        }
                    }
                }

                // Proceed along the current sector.
                current.pt = current.pt.next();
                self.stack.push(current);
                return Some((pos, current_value));
            } else {
                // Hit the end of the sector, branch ahead.
                if let Some(value) = current.group_value {
                    self.stack.push(Sector {
                        begin: current.begin.further(),
                        pt: current.begin.further(),
                        end: current.end.further(),
                        prev_value: value,
                        group_value: None,
                    });
                }

                self.next()
            }
        } else {
            None
        }
    }
}

struct Sector<T> {
    /// Start point of current sector.
    begin: PolarPoint,
    /// Point currently being processed.
    pt: PolarPoint,
    /// End point of current sector.
    end: PolarPoint,
    /// The user value from previous iteration.
    prev_value: T,
    /// The user value for this group.
    group_value: Option<T>,
}

/// Points on a hex circle expressed in polar coordinates.
#[derive(Copy, Clone, PartialEq)]
struct PolarPoint {
    pos: f32,
    radius: u32,
}

impl PolarPoint {
    pub fn new(pos: f32, radius: u32) -> PolarPoint {
        PolarPoint {
            pos: pos,
            radius: radius,
        }
    }
    /// Index of the discrete hex cell along the circle that corresponds to this point.
    fn winding_index(self) -> i32 { (self.pos + 0.5).floor() as i32 }

    pub fn is_below(self, other: PolarPoint) -> bool { self.winding_index() < other.end_index() }

    fn end_index(self) -> i32 { (self.pos + 0.5).ceil() as i32 }

    pub fn to_v2(self) -> Point2D<i32> {
        if self.radius == 0 {
            return Point2D::new(0, 0);
        }
        let index = self.winding_index();
        let sector = index.mod_floor(&(self.radius as i32 * 6)) / self.radius as i32;
        let offset = index.mod_floor(&(self.radius as i32));

        let rod = Dir6::from_int(sector).to_v2();
        let tangent = Dir6::from_int((sector + 2) % 6).to_v2();

        rod * (self.radius as i32) + tangent * offset
    }

    /// If this point and the next point are adjacent vertically (along the xy
    /// axis), return the point outside of the circle between the two points.
    ///
    /// This is a helper function for the FOV special case where acute corners
    /// of fake isometric rooms are marked visible even though strict hex FOV
    /// logic would keep them unseen.
    pub fn side_point(self) -> Option<Point2D<i32>> {
        let next = self.next();
        let a = self.to_v2();
        let b = next.to_v2();

        if b.x == a.x + 1 && b.y == a.y + 1 {
            // Going down the right rim.
            Some(Point2D::new(a.x + 1, a.y))
        } else if b.x == a.x - 1 && b.y == a.y - 1 {
            // Going up the left rim.
            Some(Point2D::new(a.x - 1, a.y))
        } else {
            None
        }
    }

    /// The point corresponding to this one on the hex circle with radius +1.
    pub fn further(self) -> PolarPoint {
        PolarPoint::new(self.pos * (self.radius + 1) as f32 / self.radius as f32,
                        self.radius + 1)
    }

    /// The point next to this one along the hex circle.
    pub fn next(self) -> PolarPoint { PolarPoint::new((self.pos + 0.5).floor() + 0.5, self.radius) }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use std::iter::FromIterator;
    use euclid::Point2D;
    use super::{FovValue, HexFov};
    use hex::HexGeom;

    #[derive(PartialEq, Eq, Clone)]
    struct Cell1 {
        range: i32,
    }

    impl FovValue for Cell1 {
        fn advance(&self, offset: Point2D<i32>) -> Option<Self> {
            if offset.hex_dist() < self.range { Some(self.clone()) } else { None }
        }
    }

    #[derive(PartialEq, Eq, Clone)]
    struct Cell2 {
        range: i32,
    }

    impl FovValue for Cell2 {
        fn advance(&self, offset: Point2D<i32>) -> Option<Self> {
            if offset.hex_dist() < self.range { Some(self.clone()) } else { None }
        }

        fn is_fake_isometric_wall(&self, offset: Point2D<i32>) -> bool {
            let _ = offset;
            true
        }
    }

    #[test]
    fn trivial_fov() {
        // Just draw a small circle.
        let field: HashMap<Point2D<i32>, Cell1> = HashMap::from_iter(HexFov::new(Cell1 {
            range: 2,
        }));
        assert!(field.contains_key(&Point2D::new(1, 0)));
        assert!(!field.contains_key(&Point2D::new(1, -1)));

        // Now test out the fake-isometric corners.
        let field: HashMap<Point2D<i32>, Cell2> = HashMap::from_iter(HexFov::new(Cell2 {
            range: 2,
        }));
        assert!(field.contains_key(&Point2D::new(1, 0)));
        assert!(field.contains_key(&Point2D::new(1, -1)));
    }
}

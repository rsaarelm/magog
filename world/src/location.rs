use std::ops::Add;
use euclid::Point2D;
use calx_alg::noise;
use calx_grid::{Dir6, HexGeom, GridNode};

/// Unambiguous location in the game world.
#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location {
        Location { x: x, y: y, z: 0 }
    }

    /// Vector pointing from this location into the other one if the locations
    /// are on the same Euclidean plane.
    pub fn v2_at(&self, other: Location) -> Option<Point2D<i32>> {
        if self.z != other.z {
            return None;
        }
        Some(Point2D::new(other.x as i32, other.y as i32) -
             Point2D::new(self.x as i32, self.y as i32))
    }

    /// Hex distance from this location to the other one, if applicable.
    pub fn distance_from(&self, other: Location) -> Option<i32> {
        if let Some(v) = self.v2_at(other) {
            Some(v.hex_dist())
        } else {
            None
        }
    }

    pub fn dir6_towards(&self, other: Location) -> Option<Dir6> {
        if let Some(v) = self.v2_at(other) {
            Some(Dir6::from_v2(v))
        } else {
            None
        }
    }

    pub fn noise(&self) -> f32 {
        noise(self.x as i32 + self.y as i32 * 59 + self.z as i32 * 919)
    }
}

impl Add<Point2D<i32>> for Location {
    type Output = Location;
    fn add(self, other: Point2D<i32>) -> Location {
        Location {
            x: (self.x as i32 + other.x) as i8,
            y: (self.y as i32 + other.y) as i8,
            z: self.z,
        }
    }
}

/// An abstract type that maps a 2D plane into game world Locations. This can
/// be just a straightforward mapping, or it can involve something exotic like
/// a non-Euclidean space where the lines from the Chart origin are raycast
/// through portals.
pub trait Chart: Add<Point2D<i32>, Output=Location> {}

impl Chart for Location {}

/// The other half of a Chart, mapping Locations into 2D plane positions, if a
/// mapping exists. It depends on the weirdness of a space how trivial this is
/// to do.
pub trait Unchart {
    fn chart_pos(&self, loc: Location) -> Option<Point2D<i32>>;
}

impl Unchart for Location {
    fn chart_pos(&self, loc: Location) -> Option<Point2D<i32>> {
        if self.z != loc.z {
            return None;
        }
        Some(Point2D::new(loc.x as i32 - self.x as i32, loc.y as i32 - self.y as i32))
    }
}

impl GridNode for Location {
    fn neighbors(&self) -> Vec<Location> {
        Dir6::iter().map(|d| *self + d.to_v2()).collect()
    }
}

#[cfg(test)]
mod test {
    use super::Location;
    use euclid::Point2D;

    #[test]
    fn test_wraparound() {
        let l1 = Location::new(0, 0);
        let l2 = l1 + Point2D::new(300, 300);
        assert_eq!((44, 44), (l2.x, l2.y));
    }
}

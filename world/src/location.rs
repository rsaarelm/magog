use std::ops::{Add, Sub};
use std::num::Wrapping;
use euclid::Point2D;
use calx_alg::noise;
use calx_grid::{Dir6, GridNode, HexGeom};

/// Unambiguous location in the game world.
#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl Location {
    pub fn new(x: i8, y: i8, z: i8) -> Location {
        Location { x: x, y: y, z: z }
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

    /// A pseudorandom value corresponding to this specific location.
    ///
    /// Is always the same for the same location value.
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

impl Add<Portal> for Location {
    type Output = Location;
    fn add(self, other: Portal) -> Location {
        Location {
            x: (Wrapping(self.x) + Wrapping(other.dx)).0,
            y: (Wrapping(self.y) + Wrapping(other.dy)).0,
            z: other.z,
        }
    }
}

impl Sub<Point2D<i32>> for Location {
    type Output = Location;
    fn sub(self, other: Point2D<i32>) -> Location {
        Location {
            x: (self.x as i32 - other.x) as i8,
            y: (self.y as i32 - other.y) as i8,
            z: self.z,
        }
    }
}

impl GridNode for Location {
    fn neighbors(&self) -> Vec<Location> {
        Dir6::iter().map(|d| *self + d.to_v2()).collect()
    }
}

#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Portal {
    pub dx: i8,
    pub dy: i8,
    pub z: i8,
}

impl Portal {
    pub fn new(from: Location, to: Location) -> Portal {
        Portal {
            dx: (Wrapping(to.x) - Wrapping(from.x)).0,
            dy: (Wrapping(to.y) - Wrapping(from.y)).0,
            z: to.z,
        }
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

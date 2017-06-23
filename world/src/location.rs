use calx_alg::{compact_bits_by_2, noise, spread_bits_by_2};
use calx_grid::{Dir6, GridNode, HexGeom};
use euclid::{Vector2D, vec2};
use std::num::Wrapping;
use std::ops::{Add, Sub};

/// Unambiguous location in the game world.
#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    pub z: i8,
}

impl Location {
    pub fn origin() -> Location { Location { x: 0, y: 0, z: 0 } }

    pub fn new(x: i8, y: i8, z: i8) -> Location { Location { x, y, z } }

    /// Construct a Location from a Morton code representation.
    ///
    /// Use the representation generated with `to_morton`. The odd low 16 bits are compacted to x
    /// value, the even low 16 bits to z and the first 8 of the high 16 bits become z.
    pub fn from_morton(morton_code: u32) -> Location {
        let xy = morton_code & 0xffff_ffff;
        let x = compact_bits_by_2(xy) as u8;
        let y = compact_bits_by_2(xy >> 1) as u8;
        let z = (morton_code >> 16) as u8;

        unsafe {
            Location {
                x: ::std::mem::transmute(x),
                y: ::std::mem::transmute(y),
                z: ::std::mem::transmute(z),
            }
        }
    }

    /// Turn the Location to a Morton code value.
    ///
    /// Spatially close locations are often numerically close in Morton codes, these are useful for
    /// quadtree-like structures.
    pub fn to_morton(&self) -> u32 {
        let mut ret = 0;
        let x: u8 = unsafe { ::std::mem::transmute(self.x) };
        let y: u8 = unsafe { ::std::mem::transmute(self.y) };
        let z: u8 = unsafe { ::std::mem::transmute(self.z) };
        ret ^= spread_bits_by_2(x as u32);
        ret ^= spread_bits_by_2(y as u32) << 1;
        ret ^= (z as u32) << 16;
        ret
    }

    /// Vector pointing from this location into the other one if the locations
    /// are on the same Euclidean plane.
    pub fn v2_at(&self, other: Location) -> Option<Vector2D<i32>> {
        if self.z != other.z {
            return None;
        }
        Some(vec2(other.x as i32, other.y as i32) -
             vec2(self.x as i32, self.y as i32))
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
    pub fn noise(&self) -> f32 { noise(self.x as i32 + self.y as i32 * 59 + self.z as i32 * 919) }
}

impl Add<Vector2D<i32>> for Location {
    type Output = Location;
    fn add(self, other: Vector2D<i32>) -> Location {
        Location {
            x: (self.x as i32 + other.x) as i8,
            y: (self.y as i32 + other.y) as i8,
            z: self.z,
        }
    }
}

impl Add<Dir6> for Location {
    type Output = Location;
    fn add(self, other: Dir6) -> Location { self + other.to_v2() }
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

impl Sub<Vector2D<i32>> for Location {
    type Output = Location;
    fn sub(self, other: Vector2D<i32>) -> Location {
        Location {
            x: (self.x as i32 - other.x) as i8,
            y: (self.y as i32 - other.y) as i8,
            z: self.z,
        }
    }
}

impl GridNode for Location {
    fn neighbors(&self) -> Vec<Location> { Dir6::iter().map(|d| *self + d.to_v2()).collect() }
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

impl Add<Portal> for Portal {
    type Output = Portal;
    fn add(self, other: Portal) -> Portal {
        Portal {
            dx: (Wrapping(self.dx) + Wrapping(other.dx)).0,
            dy: (Wrapping(self.dy) + Wrapping(other.dy)).0,
            z: other.z,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Location;
    use euclid::vec2;

    #[test]
    fn test_wraparound() {
        let l1 = Location::new(0, 0, 0);
        let l2 = l1 + vec2(300, 300);
        assert_eq!((44, 44), (l2.x, l2.y));
    }

    #[test]
    fn test_morton() {
        use rand::{self, Rng};
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let x = rng.gen::<u32>() & 0xff_ffff;
            assert_eq!(x, Location::from_morton(x).to_morton());
        }
    }
}

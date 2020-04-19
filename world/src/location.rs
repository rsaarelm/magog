#![allow(clippy::cast_lossless)]

use crate::{sector::Sector, World};
use calx::{
    compact_bits_by_2, hex_neighbors, spread_bits_by_2, CellSpace, CellVector, Dir6, GridNode,
    HexGeom, ProjectVec,
};
use euclid::{vec2, Vector2D};
use serde_derive::{Deserialize, Serialize};
use std::num::Wrapping;
use std::ops::{Add, Sub};

/// Unambiguous location in the game world.
#[derive(
    Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Default, Serialize, Deserialize,
)]
pub struct Location {
    pub x: i16,
    pub y: i16,
    pub z: i16,
}

/// The type for a unique location in the game world.
///
/// IMPORTANT: Be careful where you use the simple "location + vec" algebra. That does not take
/// portals into account, and will usually cause unwanted effects near them in high-level code.
/// Most high-level logic should use `Location::jump` to displace locations. This will correctly
/// traverse portals.
impl Location {
    pub fn origin() -> Location { Location { x: 0, y: 0, z: 0 } }

    pub fn new(x: i16, y: i16, z: i16) -> Location { Location { x, y, z } }

    /// Construct a Location from a Morton code representation.
    ///
    /// Use the representation generated with `to_morton`. The odd bits of the low 32 bits are
    /// compacted to x value, the even bits of the low 32 bits to y and the first 16 of the high 32
    /// bits become z.
    pub fn from_morton(morton_code: u64) -> Location {
        let xy = (morton_code & 0xffff_ffff) as u32;
        let x = compact_bits_by_2(xy) as u16;
        let y = compact_bits_by_2(xy >> 1) as u16;
        let z = (morton_code >> 32) as u16;

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
    pub fn to_morton(self) -> u64 {
        let mut ret = 0;
        let x: u16 = unsafe { ::std::mem::transmute(self.x) };
        let y: u16 = unsafe { ::std::mem::transmute(self.y) };
        let z: u16 = unsafe { ::std::mem::transmute(self.z) };
        ret ^= spread_bits_by_2(x as u32) as u64;
        ret ^= (spread_bits_by_2(y as u32) << 1) as u64;
        ret ^= (z as u64) << 32;
        ret
    }

    /// Vector pointing from this location into the other one if the locations
    /// are on the same Euclidean plane.
    pub fn v2_at(self, other: Location) -> Option<CellVector> {
        if self.z != other.z {
            return None;
        }
        Some(vec2(other.x as i32, other.y as i32) - vec2(self.x as i32, self.y as i32))
    }

    /// Hex distance from this location to the other one, if applicable.
    pub fn distance_from(self, other: Location) -> Option<i32> {
        if let Some(v) = self.v2_at(other) {
            Some(v.hex_dist())
        } else {
            None
        }
    }

    /// Distance that defaults to max integer value for separate zones.
    ///
    /// Can be used for situations that want a straightforward metric function like A* search.
    pub fn metric_distance(self, other: Location) -> i32 {
        self.distance_from(other).unwrap_or(i32::max_value())
    }

    pub fn dir6_towards(self, other: Location) -> Option<Dir6> {
        if let Some(v) = self.v2_at(other) {
            Some(Dir6::from_v2(v))
        } else {
            None
        }
    }

    /// Offset location and follow any portals in target site.
    pub fn jump<V: Into<CellVector> + Sized>(self, ctx: &World, offset: V) -> Location {
        let loc = self + offset.into();
        ctx.portal(loc).unwrap_or(loc)
    }

    /// True for a one-cell wide border region between sectors
    pub fn on_sector_border(self) -> bool {
        let sec = Sector::from(self);
        // Only test along three adjacent directions. This way we get a 1-cell wide border
        // everywhere. Testing the full circle would produce a 2-cell wide border.
        hex_neighbors(self)
            .take(3)
            .map(Sector::from)
            .any(|s| s != sec)
    }

    /// Smooth noise offset for determinining overland cell boundaries at this location.
    pub fn terrain_cell_displacement(self) -> CellVector {
        use lazy_static::lazy_static;
        use noise::NoiseFn;
        lazy_static! {
            static ref NOISE: noise::OpenSimplex = noise::OpenSimplex::new();
        }

        let (dx, dy) = {
            const ZOOM: f64 = 1.0 / 2.0;
            const SCALE: f64 = 4.0;
            let (x, y) = (self.x as f64 * ZOOM, self.y as f64 * ZOOM);
            // Use 3D hex coordinates to get a symmetric kernel.
            // Sample the different components from different places in the noise plane.
            let dx = SCALE * NOISE.get([x, y]);
            let dy = SCALE * NOISE.get([x + 6553.0, y + 9203.0]);
            let dz = SCALE * NOISE.get([x + 9203.0, y + 6553.0]);
            (dx + dz, dy + dz)
        };
        vec2(dx.round() as i32, dy.round() as i32)
    }
}

impl From<Location> for CellVector {
    fn from(loc: Location) -> Self { vec2(loc.x as i32, loc.y as i32) }
}

impl From<Sector> for Location {
    fn from(sec: Sector) -> Self {
        Location::new(0, 0, sec.z) + Vector2D::from(sec).project::<CellSpace>()
    }
}

impl<V: Into<CellVector>> Add<V> for Location {
    type Output = Location;
    fn add(self, other: V) -> Location {
        let other = other.into();
        Location {
            x: (self.x as i32 + other.x) as i16,
            y: (self.y as i32 + other.y) as i16,
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

impl<V: Into<CellVector>> Sub<V> for Location {
    type Output = Location;
    fn sub(self, other: V) -> Location {
        let other = other.into();
        Location {
            x: (self.x as i32 - other.x) as i16,
            y: (self.y as i32 - other.y) as i16,
            z: self.z,
        }
    }
}

impl GridNode for Location {
    fn neighbors(&self) -> Vec<Location> { hex_neighbors(*self).collect() }
}

#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, Serialize, Deserialize)]
pub struct Portal {
    pub dx: i16,
    pub dy: i16,
    pub z: i16,
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
    use crate::sector::Sector;
    use calx::{CellSpace, CellVector, ProjectVec, StaggeredHexSpace};
    use euclid::vec2;

    #[test]
    fn test_wraparound() {
        let l1 = Location::new(0, 0, 0);
        let l2 = l1 + vec2(66000, 66000);
        assert_eq!((464, 464), (l2.x, l2.y));
    }

    #[test]
    fn test_morton() {
        use rand::Rng;
        let mut rng = rand::thread_rng();

        for _ in 0..1000 {
            let x = rng.gen::<u64>() & 0xffff_ffff_ffff;
            assert_eq!(x, Location::from_morton(x).to_morton());
        }
    }

    #[test]
    fn test_location_to_sector() {
        // Sector division near origin
        assert_eq!(Sector::from(Location::new(0, 0, 0)), Sector::new(0, 0, 0));
        assert_eq!(Sector::from(Location::new(0, 1, 0)), Sector::new(-1, 0, 0));
        assert_eq!(
            Sector::from(Location::new(-1, 0, 0)),
            Sector::new(-1, -1, 0)
        );

        for y in -100..100 {
            for x in -100..100 {
                let loc = Location::new(x, y, 0);
                let vec = CellVector::from(loc);
                assert_eq!(
                    vec,
                    vec.project::<StaggeredHexSpace>().project::<CellSpace>()
                );

                assert!(
                    Sector::from(loc).iter().find(|&x| x == loc).is_some(),
                    format!("{:?} not found in sector {:?}", loc, Sector::from(loc))
                );
            }
        }
    }

    #[test]
    fn test_sector_iter() {
        let s = Sector::new(0, 0, 0);

        for loc in s.iter() {
            assert_eq!(s, Sector::from(loc), "Location: {:?}", loc);
        }
    }
}

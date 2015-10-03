use std::ops::Add;
use calx::{V2, Dir6, HexGeom, LatticeNode, noise};
use content::TerrainType;

/// Unambiguous location in the game world.
#[derive(Copy, Eq, PartialEq, Clone, Hash, PartialOrd, Ord, Debug, RustcEncodable, RustcDecodable)]
pub struct Location {
    pub x: i8,
    pub y: i8, // TODO: Add third dimension for multiple persistent levels.
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location {
        Location { x: x, y: y }
    }

    /// Vector pointing from this location into the other one if the locations
    /// are on the same Euclidean plane.
    pub fn v2_at(&self, other: Location) -> Option<V2<i32>> {
        // Return None for pairs on different floors if multi-floor support is
        // added.
        Some(V2(other.x as i32, other.y as i32) - V2(self.x as i32, self.y as i32))
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
        noise(self.x as i32 + self.y as i32 * 57)
    }
}

impl Add<V2<i32>> for Location {
    type Output = Location;
    fn add(self, other: V2<i32>) -> Location {
        Location::new((self.x as i32 + other.0) as i8,
                      (self.y as i32 + other.1) as i8)
    }
}

/// An abstract type that maps a 2D plane into game world Locations. This can
/// be just a straightforward mapping, or it can involve something exotic like
/// a non-Euclidean space where the lines from the Chart origin are raycast
/// through portals.
pub trait Chart: Add<V2<i32>, Output=Location> {}

impl Chart for Location {}

/// The other half of a Chart, mapping Locations into 2D plane positions, if a
/// mapping exists. It depends on the weirdness of a space how trivial this is
/// to do.
pub trait Unchart {
    fn chart_pos(&self, loc: Location) -> Option<V2<i32>>;
}

impl Unchart for Location {
    fn chart_pos(&self, loc: Location) -> Option<V2<i32>> {
        Some(V2(loc.x as i32 - self.x as i32,
                loc.y as i32 - self.y as i32))
    }
}

impl LatticeNode for Location {
    fn neighbors(&self) -> Vec<Location> {
        Dir6::iter().map(|d| *self + d.to_v2()).collect()
    }
}

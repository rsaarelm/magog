use calx::dijkstra;
use calx::V2;
use dir6::Dir6;
use entity::Entity;
use terrain::TerrainType;
use geom::HexGeom;
use terrain;
use world;

/// Unambiguous location in the game world.
#[deriving(Eq, PartialEq, Clone, Hash, Show, Encodable, Decodable)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    // TODO: Add third dimension for multiple persistent levels.
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location { Location { x: x, y: y } }

    /// Return terrain at the location.
    pub fn terrain(&self) -> TerrainType {
        let mut ret = world::get().borrow().area.terrain(*self);
        // Mobs standing on doors make the doors open.
        if ret == terrain::Door && self.has_mobs() {
            ret = terrain::OpenDoor;
        }
        ret
    }

    pub fn blocks_sight(&self) -> bool {
        self.terrain().blocks_sight()
    }

    pub fn blocks_walk(&self) -> bool {
        if self.terrain().blocks_walk() { return true; }
        if self.entities().iter().any(|e| e.blocks_walk()) {
            return true;
        }
        false
    }

    pub fn entities(&self) -> Vec<Entity> {
        world::get().borrow().spatial.entities_at(*self)
    }

    pub fn has_entities(&self) -> bool { !self.entities().is_empty() }

    pub fn has_mobs(&self) -> bool {
        self.entities().iter().any(|e| e.is_mob())
    }

    /// Vector pointing from this location into the other one if the locations
    /// are on the same Euclidean plane.
    pub fn v2_at(&self, other: Location) -> Option<V2<int>> {
        // Return None for pairs on different floors if multi-floor support is
        // added.
        Some(V2(other.x as int, other.y as int) - V2(self.x as int, self.y as int))
    }

    /// Hex distance from this location to the other one, if applicable.
    pub fn distance_from(&self, other: Location) -> Option<int> {
        if let Some(v) = self.v2_at(other) { Some(v.hex_dist()) } else { None }
    }

    pub fn dir6_towards(&self, other: Location) -> Option<Dir6> {
        if let Some(v) = self.v2_at(other) { Some(Dir6::from_v2(v)) } else { None }
    }
}

impl Add<V2<int>, Location> for Location {
    fn add(&self, other: &V2<int>) -> Location {
        Location::new(
            (self.x as int + other.0) as i8,
            (self.y as int + other.1) as i8)
    }
}

/// An abstract type that maps a 2D plane into game world Locations. This can
/// be just a straightforward mapping, or it can involve something exotic like
/// a non-Euclidean space where the lines from the Chart origin are raycast
/// through portals.
pub trait Chart: Add<V2<int>, Location> {}

impl Chart for Location {}

/// The other half of a Chart, mapping Locations into 2D plane positions, if a
/// mapping exists. It depends on the weirdness of a space how trivial this is
/// to do.
pub trait Unchart {
    fn chart_pos(&self, loc: Location) -> Option<V2<int>>;
}

impl Unchart for Location {
    fn chart_pos(&self, loc: Location) -> Option<V2<int>> {
        Some(V2(loc.x as int - self.x as int, loc.y as int - self.y as int))
    }
}

impl dijkstra::Node for Location {
    fn neighbors(&self) -> Vec<Location> {
        Dir6::iter().map(|d| *self + d.to_v2()).collect()
    }
}

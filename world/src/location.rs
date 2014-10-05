use calx::V2;
use ecs::Entity;
use terrain::TerrainType;

/// Unambiguous location in the game world.
#[deriving(Eq, PartialEq, Clone, Hash, Show)]
pub struct Location {
    pub x: i8,
    pub y: i8,
    // TODO: Add third dimension for multiple persistent levels.
}

impl Location {
    pub fn new(x: i8, y: i8) -> Location { Location { x: x, y: y } }

    /// Return terrain at the location.
    pub fn terrain(&self) -> TerrainType { unimplemented!(); }

    /// Set the terrain at the location. None will reset to default terrain.
    pub fn set_terrain(&self, t: Option<TerrainType>) { unimplemented!(); }

    pub fn blocks_sight(&self) -> bool { unimplemented!(); }
    pub fn blocks_walk(&self) -> bool { unimplemented!(); }
    pub fn entities(&self) -> Vec<Entity> { unimplemented!(); }
    pub fn has_entities(&self) -> bool { !self.entities().is_empty() }
}


impl Add<V2<int>, Location> for Location {
    fn add(&self, other: &V2<int>) -> Location {
        Location::new(
            (self.x as int + other.0) as i8,
            (self.y as int + other.1) as i8)
    }
}

impl Sub<Location, V2<int>> for Location {
    fn sub(&self, other: &Location) -> V2<int> {
        V2((self.x - other.x) as int, (self.y - other.y) as int)
    }
}

/// An abstract type that maps a 2D plane into game world Locations. This can
/// be just a straightforward mapping, or it can involve something exotic like
/// a non-Euclidean space where the lines from the Chart origin are raycast
/// through portals.
pub trait Chart: Add<V2<int>, Location> {}

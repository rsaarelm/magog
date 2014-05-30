use world::terrain::TerrainType;
use world::terrain;
use world::world::{World, Location};

pub trait Area {
    fn terrain_at(&self, loc: Location) -> TerrainType;

    /// Return the terrain type a location with no explicitly specified terrain
    /// will have, based on some simple formula. Eg. overland default terrain
    /// is always ocean, underground default terrain is always solid rock.
    fn default_terrain_at(&self, loc: Location) -> TerrainType;

    /// Return whether a terrain cell can be seen through.
    fn is_opaque(&self, loc: Location) -> bool;
}

impl Area for World {
    fn terrain_at(&self, loc: Location) -> TerrainType {
        match self.terrain_get(loc) {
            Some(t) => t,
            None => self.default_terrain_at(loc)
        }
    }

    fn default_terrain_at(&self, _loc: Location) -> TerrainType {
        // TODO: Logic for this
        terrain::Void
    }

    fn is_opaque(&self, loc: Location) -> bool {
        // This is a separate method since non-terrain elements may end up
        // contributing to this.
        self.terrain_at(loc).is_opaque()
    }
}

use world::terrain::TerrainType;
use world::terrain;
use world::world::{World, Location};
use world::mobs::Mobs;

pub trait Area {
    fn terrain_at(&self, loc: Location) -> TerrainType;

    /// Return the terrain type a location with no explicitly specified terrain
    /// will have, based on some simple formula. Eg. overland default terrain
    /// is always ocean, underground default terrain is always solid rock.
    fn default_terrain_at(&self, loc: Location) -> TerrainType;

    /// Return whether a terrain cell blocks visibility
    fn is_opaque(&self, loc: Location) -> bool;

    /// Return whether a cell blocks movement.
    fn is_walkable(&self, loc: Location) -> bool;

    fn open_locs(&self) -> Vec<Location>;
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

    fn is_walkable(&self, loc: Location) -> bool {
        if !self.terrain_at(loc).is_walkable() {
            return false;
        }

        self.mobs_at(loc).len() == 0
    }

    fn open_locs(&self) -> Vec<Location> {
        self.area.keys()
            .filter(|&&loc| self.is_walkable(loc))
            .map(|&loc| loc).collect()
    }
}

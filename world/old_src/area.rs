use cgmath::Vector2;
use terrain::*;
use system::{Entity, World};
use spatial::{Location};

/// Terrain properties and operations.
pub trait Area {
    fn terrain_get(&self, loc: Location) -> Option<TerrainType>;

    fn terrain_set(&mut self, loc: Location, t: TerrainType);

    fn terrain_clear(&mut self, loc: Location);

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

    fn entities_at(&self, loc: Location) -> Vec<Entity>;

    fn has_entities_at(&self, loc: Location) -> bool {
        !self.entities_at(loc).is_empty()
    }

    fn open_neighbors<'a, T: Iterator<&'a Vector2<int>>>(&self, loc: Location, dirs: T) -> Vec<Location> {
        dirs.map(|&d| loc + d)
            .filter(|&loc| self.is_walkable(loc))
            .collect()
    }
}

impl Area for World {
    fn terrain_get(&self, loc: Location) -> Option<TerrainType> {
        self.system().area.find(&loc).map(|x| *x)
    }

    fn terrain_set(&mut self, loc: Location, t: TerrainType) {
        self.system_mut().area.insert(loc, t);
    }

    fn terrain_clear(&mut self, loc: Location) {
        self.system_mut().area.remove(&loc);
    }

    fn terrain_at(&self, loc: Location) -> TerrainType {
        let mut ret = match self.terrain_get(loc) {
            Some(t) => t,
            None => self.default_terrain_at(loc)
        };

        // Make doors open if someone is walking through them.
        if ret == Door && self.has_entities_at(loc) {
            ret = OpenDoor;
        }

        ret
    }

    fn default_terrain_at(&self, _loc: Location) -> TerrainType {
        if self.system().depth == 1 {
            // Overworld
            Tree
        } else {
            // Underground
            Rock
        }
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

        !self.has_entities_at(loc)
    }

    fn open_locs(&self) -> Vec<Location> {
        self.system().area.keys()
            .filter(|&&loc| self.is_walkable(loc))
            .map(|&loc| loc).collect()
    }

    fn entities_at(&self, loc: Location) -> Vec<Entity> {
        self.system().spatial.entities_at(loc)
    }
}

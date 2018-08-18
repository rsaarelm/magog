use crate::location::{Location, Portal};
use crate::terrain::{self, Terrain};

/// Immutable terrain field query.
pub trait TerrainQuery {
    /// Return whether location is contained in the current play area.
    fn is_valid_location(&self, loc: Location) -> bool;

    /// Return terrain at location.
    fn terrain(&self, loc: Location) -> Terrain;

    /// If location contains a portal, return the destination of the portal.
    fn portal(&self, loc: Location) -> Option<Location>;

    /// Return whether location has a border portals.
    ///
    /// Portals are divided into border and hole portals. A hole portal is usually surrounded by
    /// local scenery at all sides, while border portals are assumed to be at the edges of a convex
    /// patch of local terrain with nothing local past them.
    ///
    /// The difference between the two is important in how map memory works, map memory display
    /// goes through border portals normally, but ignores hole portals.
    fn is_border_portal(&self, _loc: Location) -> bool {
        // TODO: Implement internal data for border portals
        // Turn type Portal in the internal portal data into enum { Border(Portal), Edge(Portal) }
        false
    }

    /// The cell has not (probably) been touched by map generation yet.
    fn is_untouched(&self, loc: Location) -> bool;

    /// Return a portal if it can be seen through.
    fn visible_portal(&self, loc: Location) -> Option<Location> {
        // Only void-form is transparent to portals.
        if self.terrain(loc).form() == terrain::Form::Void {
            self.portal(loc)
        } else {
            None
        }
    }
}

/// Methods to modify world terrain.
pub trait Terraform {
    /// Set map terrain.
    ///
    /// A zero value will generally reset the terrain to the map default.
    fn set_terrain(&mut self, loc: Location, terrain: terrain::Terrain);

    /// Set a portal on map.
    ///
    /// If the portal points to a location with an existing portal, the portal value will be
    /// modified to point to that portal's destination.
    ///
    /// If the portal does not involve any translation, it will not be added.
    fn set_portal(&mut self, loc: Location, portal: Portal);

    /// Remove any portals from given location.
    fn remove_portal(&mut self, loc: Location);
}

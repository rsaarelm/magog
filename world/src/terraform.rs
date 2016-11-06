use std::sync::Arc;
use location::{Location, Portal};
use terrain;

/// Immutable terrain field query.
pub trait TerrainQuery {
    /// Return whether location is contained in the current play area.
    fn is_valid_location(&self, loc: Location) -> bool;

    /// Return terrain at location.
    fn terrain(&self, loc: Location) -> Arc<terrain::Tile>;

    /// If location contains a portal, return the destination of the portal.
    fn portal(&self, loc: Location) -> Option<Location>;

    /// Return a portal if it can be seen through.
    fn visible_portal(&self, loc: Location) -> Option<Location> {
        // Only void-form is transparent to portals.
        if self.terrain(loc).form == terrain::Form::Void { self.portal(loc) } else { None }
    }
}

/// Methods to modify world terrain.
pub trait Terraform {
    /// Set map terrain.
    ///
    /// A zero value will generally reset the terrain to the map default.
    fn set_terrain(&mut self, loc: Location, terrain: u8);

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

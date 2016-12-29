use location::{Location, Portal};
use terrain::{self, Terrain};

/// Immutable terrain field query.
pub trait TerrainQuery {
    /// Return whether location is contained in the current play area.
    fn is_valid_location(&self, loc: Location) -> bool;

    /// Return terrain at location.
    fn terrain(&self, loc: Location) -> Terrain;

    /// If location contains a portal, return the destination of the portal.
    fn portal(&self, loc: Location) -> Option<Location>;

    /// The cell has not (probably) been touched by map generation yet.
    fn is_untouched(&self, loc: Location) -> bool;

    /// Return a portal if it can be seen through.
    fn visible_portal(&self, loc: Location) -> Option<Location> {
        // Only void-form is transparent to portals.
        if self.terrain(loc).form() == terrain::Form::Void { self.portal(loc) } else { None }
    }

    /// Does this location belong to a placed room in mapgen.
    fn belongs_to_a_room(&self, loc: Location) -> bool {
        // XXX: This is dodgy and assumes dungeony maps that start out as undug blocks, plus
        // assumes that room terrain will not consist of blocks. May want to have more
        // sophisticated logic here in the future.
        match self.terrain(loc).kind() {
            terrain::Kind::Block | terrain::Kind::Corridor => false,
            _ => true
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

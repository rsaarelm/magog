use location::{Location, Portal};

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

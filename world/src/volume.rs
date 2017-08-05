use calx_grid::HexFov;
use fov::SphereVolumeFov;
use location::Location;
use std::iter::FromIterator;
use world::World;

/// `Volume` is a specific area of the game world.
pub struct Volume(pub Vec<Location>);

impl Volume {
    /// Create a volume that consists of a single point.
    pub fn point(loc: Location) -> Volume { Volume(vec![loc]) }

    /// Construct a sphere volume that follows portals and is stopped by walls.
    ///
    /// The stopping walls are terrain for which `blocks_shot` is true.
    pub fn sphere(w: &World, origin: Location, radius: u32) -> Volume {
        // TODO: Add stop predicate to API, allow passing through walls.
        Volume(Vec::from_iter(
            HexFov::new(SphereVolumeFov::new(w, radius, origin)).map(
                |(pos, a)| a.origin + pos,
            ),
        ))
    }
}

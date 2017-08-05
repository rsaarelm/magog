use calx_grid::HexFov;
use fov::SphereVolumeFov;
use location::Location;
use std::iter::FromIterator;
use world::World;

/// `Volume` is a specific area of the game world.
pub struct Volume(pub Vec<Location>);

impl Volume {
    pub fn point(loc: Location) -> Volume { Volume(vec![loc]) }

    pub fn sphere(w: &World, origin: Location, radius: u32) -> Volume {
        Volume(Vec::from_iter(
            HexFov::new(SphereVolumeFov::new(w, radius, origin)).map(
                |(pos, a)| a.origin + pos,
            ),
        ))
    }
}

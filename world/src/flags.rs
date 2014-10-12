use location::Location;
use world;

#[deriving(Encodable, Decodable)]
pub struct Flags {
    pub seed: u32,
    pub camera: Location,
}

impl Flags {
    pub fn new(seed: u32) -> Flags {
        Flags {
            seed: seed,
            camera: Location::new(0, 0),
        }
    }
}

/// Get the location where the current view should be centered.
pub fn camera() -> Location {
    world::get().borrow().flags.camera
}

/// Move the current view location.
pub fn set_camera(loc: Location) {
    world::get().borrow_mut().flags.camera = loc;
}

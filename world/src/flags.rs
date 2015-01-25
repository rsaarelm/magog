use location::Location;
use world;

#[derive(Copy, RustcEncodable, RustcDecodable)]
pub struct Flags {
    pub seed: u32,
    pub camera: Location,
    pub tick: u64,
    pub player_acted: bool,
}

impl Flags {
    pub fn new(seed: u32) -> Flags {
        Flags {
            seed: seed,
            camera: Location::new(0, 0, 0),
            tick: 0,
            player_acted: false,
        }
    }
}

/// Get the location where the current view should be centered.
pub fn camera() -> Location {
    world::with(|w| w.flags.camera)
}

/// Move the current view location.
pub fn set_camera(loc: Location) {
    world::with_mut(|w| w.flags.camera = loc);
}

/// Return the frame count since the start of the game.
pub fn get_tick() -> u64 {
    world::with(|w| w.flags.tick)
}

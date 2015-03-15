use rand::{XorShiftRng, SeedableRng};
use location::Location;
use util::EncodeRng;
use entity::Entity;
use world;

#[derive(RustcEncodable, RustcDecodable)]
pub struct Flags {
    pub seed: u32,
    pub camera: Location,
    pub tick: u64,
    pub player_acted: bool,
    /// Store the player entity here for fast access.
    pub player: Option<Entity>,
    pub rng: EncodeRng<XorShiftRng>,
    pub terrans_left: u32,
}

impl Flags {
    pub fn new(seed: u32) -> Flags {
        Flags {
            seed: seed,
            camera: Location::new(0, 0),
            tick: 0,
            player_acted: false,
            player: None,
            rng: SeedableRng::from_seed([seed, seed, seed, seed]),
            terrans_left: 0,
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

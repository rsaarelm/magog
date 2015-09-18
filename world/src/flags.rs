use rand::{XorShiftRng, SeedableRng, Rng};
use location::Location;
use calx::EncodeRng;
use calx_ecs::Entity;
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
        }
    }
}

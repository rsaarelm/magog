use rand::{XorShiftRng, SeedableRng};
use location::Location;
use calx::EncodeRng;
use calx_ecs::Entity;

#[derive(Serialize, Deserialize)]
pub struct Flags {
    pub seed: u32,
    pub camera: Location,
    pub tick: u64,
    pub player_acted: bool,
    /// Store the player entity here for fast access.
    pub player: Option<Entity>,
    pub rng: EncodeRng<XorShiftRng>,
    pub depth: i32,
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
            depth: 0,
        }
    }
}

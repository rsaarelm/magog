
use Rng;
use calx_ecs::Entity;
use location::Location;
use rand::SeedableRng;

#[derive(Serialize, Deserialize)]
pub struct Flags {
    pub camera: Location,
    pub tick: u64,
    pub player_acted: bool,
    /// Store the player entity here for fast access.
    pub player: Option<Entity>,
    pub depth: i32,

    pub seed: u32,
    rng: Rng,
}

impl Flags {
    pub fn new(seed: u32) -> Flags {
        Flags {
            camera: Location::new(0, 0, 0),
            tick: 0,
            player_acted: false,
            player: None,
            depth: 0,

            seed,
            rng: SeedableRng::from_seed([seed, seed, seed, seed]),
        }
    }

    pub fn seed(&self) -> u32 { self.seed }
    pub fn rng(&mut self) -> &mut Rng { &mut self.rng }
}

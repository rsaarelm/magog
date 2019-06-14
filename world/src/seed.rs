use serde_derive::{Deserialize, Serialize};

/// Structure used to specify a new game world.
#[derive(Clone, Serialize, Deserialize)]
pub struct Seed {
    pub rng_seed: u32,
}

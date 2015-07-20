use rand::{XorShiftRng, SeedableRng};
use calx::{EncodeRng};

#[derive(RustcEncodable, RustcDecodable)]
pub struct Globals {
    pub rng: EncodeRng<XorShiftRng>,
}

impl Globals {
    pub fn new(seed: u32) -> Globals {
        Globals {
            rng: SeedableRng::from_seed([seed, seed, seed, seed]),
        }
    }
}

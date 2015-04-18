use rand::{Rng, XorShiftRng};
use calx::EncodeRng;
use world;

/// Execute a closure with the world RNG.
pub fn with<A, F>(f: F) -> A
    where F: Fn(&mut EncodeRng<XorShiftRng>) -> A {
    world::with_mut(|w| f(&mut w.flags.rng))
}

/// Return a floating-point value between 0 and 1.
pub fn unit() -> f64 {
    with(|ref mut rng| {
        let p: f64 = rng.gen();
        p % 1.0
    })
}

/// Returns true with probability prob.
pub fn p(prob: f64) -> bool { unit() < prob }

pub fn one_chance_in(x: u32) -> bool { unit() * (x as f64) < 1.0 }

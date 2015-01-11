use std::rand;
use std::rand::TaskRng;
use std::rand::Rng;

/// Execute a closure with the world RNG.
pub fn with<A, F>(f: F) -> A
    where F: Fn(&mut TaskRng) -> A {
    // TODO: Have a persistent seedable RNG in Flags that is used here. Will
    // need serialization of Rng state figured out.
    f(&mut rand::task_rng())
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

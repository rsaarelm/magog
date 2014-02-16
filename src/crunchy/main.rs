extern crate calx;
use calx::text;
use std::num::log10;
use std::rand::Rng;
use std::rand;

pub fn probToDb(prob: f64) -> f64 {
    assert!(prob > 0.0);
    assert!(prob < 1.0);
    -10.0 * log10(1.0 / prob - 1.0)
}

pub fn randomDb<R: Rng>(rng: &mut R) -> f64 {
    probToDb(rng.gen_range(1.0, 99.0) / 100.0)
}

pub fn main() {
    println!("{}\n",
        text::wrap_lines(20, "Crunchy: A program for testing rules and object interactions."));
    println!("{}\n", randomDb(&mut rand::task_rng()));
}

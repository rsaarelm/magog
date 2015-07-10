extern crate calx;
use calx::backend::{Mixer};

fn main() {
    println!("Starting noise");
    let mut mixer = Mixer::new();
    mixer.add_wave(Box::new(|t| (t * 3000.0).sin()), 2.0);

    mixer.run();
}

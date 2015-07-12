extern crate calx;
use std::thread;
use calx::backend::{Mixer};

fn main() {
    println!("Starting noise");
    let mut mixer = Mixer::new();
    mixer.add_wave(Box::new(|t| (t * 3000.0).sin()), 2.0);
    thread::sleep_ms(1000);
    mixer.add_wave(Box::new(|t| (t * 1000.0).sin()), 2.0);
    thread::sleep_ms(3000);
}

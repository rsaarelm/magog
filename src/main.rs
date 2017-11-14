// Don't show a console window when running on Windows.
#![windows_subsystem = "windows"]

extern crate rand;
extern crate euclid;
extern crate scancode;
extern crate vitral;
#[macro_use]
extern crate calx;
extern crate world;
extern crate display;
extern crate glium;

pub mod game_loop;

use display::Backend;
use game_loop::GameLoop;
use rand::Rng;
use world::World;

pub fn main() {
    let mut backend = Backend::start(640, 360, "Magog").expect("Failed to start rendering backend");

    let seed = rand::thread_rng().gen();
    // Print out the seed in case worldgen has a bug and we want to debug stuff with the same seed.
    println!("Seed: {}", seed);

    let mut game = GameLoop::new(&mut backend, World::new(seed));

    while game.draw(&mut backend) {}
}

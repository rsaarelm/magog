// Don't show a console window when running on Windows.
#![windows_subsystem = "windows"]

#[macro_use]
extern crate calx;
extern crate display;
extern crate env_logger;
extern crate euclid;
extern crate glium;
extern crate rand;
extern crate scancode;
extern crate structopt;
extern crate structopt_derive;
extern crate vitral;
extern crate world;

pub mod game_loop;

use calx::TimestepLoop;
use display::Backend;
use game_loop::GameLoop;
use rand::Rng;
use std::thread;
use std::time::Duration;
use structopt::StructOpt;
use world::World;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long = "seed")]
    seed: Option<u32>,
}

pub fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    const FPS: f64 = 30.0;

    env_logger::init();

    let seed = opt.seed.unwrap_or_else(|| rand::thread_rng().gen());
    // Print out the seed in case worldgen has a bug and we want to debug stuff with the same seed.
    println!("Seed: {}", seed);
    let world = World::new(seed);

    let mut timestep = TimestepLoop::new(1.0 / FPS);
    let mut backend = Backend::start(640, 360, "Magog").expect("Failed to start rendering backend");
    let mut game = GameLoop::new(&mut backend, world);

    'gameloop: loop {
        while timestep.should_update() {
            if !game.draw(&mut backend) {
                break 'gameloop;
            }
        }

        let free_time = timestep.time_until_update();
        if free_time > 0.0 {
            thread::sleep(Duration::from_millis((free_time * 1000.0) as u64));
        }

        timestep.observe_render();
    }
}

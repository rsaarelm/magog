// Don't show a console window when running on Windows.
#![windows_subsystem = "windows"]

use crate::game_loop::GameLoop;
use display;
use env_logger;
use log::info;
use rand;
use rand::Rng;
use structopt;
use structopt::StructOpt;
use vitral::{self, AppConfig, Flick};

pub mod game_loop;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(long = "seed")]
    seed: Option<u32>,
}

pub fn main() {
    let opt = Opt::from_args();

    const FPS: f64 = 30.0;
    env_logger::init();

    display::load_graphics();

    let seed = opt.seed.unwrap_or_else(|| rand::thread_rng().gen());
    // Print out the seed in case worldgen has a bug and we want to debug stuff with the same seed.
    info!("World seed: {}", seed);

    vitral::run_app(
        AppConfig::new(format!("Magog v{}", env!("CARGO_PKG_VERSION")))
            .frame_duration(Flick::from_seconds(1.0 / FPS)),
        game_loop::GameRuntime::new(seed),
        vec![Box::new(GameLoop::default())],
    )
    .unwrap();
}

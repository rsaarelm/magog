// Don't show a console window when running on Windows.
#![windows_subsystem = "windows"]

use crate::game_loop::GameLoop;
use log::info;
use rand::Rng;
use structopt::StructOpt;
use vitral::{self, AppConfig, Flick};
use world::{ExternalEntity, WorldSeed, WorldSkeleton};

pub mod game_loop;
mod msg;

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

    msg::register();

    let rng_seed = opt.seed.unwrap_or_else(|| rand::thread_rng().gen());
    // Print out the seed in case worldgen has a bug and we want to debug stuff with the same seed.
    info!("World seed: {}", rng_seed);

    let world_seed = WorldSeed {
        rng_seed,
        world_skeleton: WorldSkeleton::overworld_sprawl(),
        player_character: ExternalEntity::from_name("player").unwrap(),
    };

    vitral::App::new(
        AppConfig::new(format!("Magog v{}", env!("CARGO_PKG_VERSION")))
            .frame_duration(Flick::from_seconds(1.0 / FPS)),
        game_loop::GameRuntime::new(world_seed),
        vec![Box::new(GameLoop::default())],
    )
    .run()
}

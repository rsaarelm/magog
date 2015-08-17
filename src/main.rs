#![crate_name="magog"]

extern crate rand;
extern crate rustc_serialize;
extern crate num;
extern crate getopts;
extern crate image;
extern crate time;
extern crate toml;
extern crate calx;
extern crate mapgen;
extern crate world;

use std::env;
use std::process;
use std::path::{PathBuf};
use std::fs::{self, File};
use std::io::{Write};
use std::default::Default;
// TODO: Get a stable standard library PathExt to replace this.
use calx::{PathExt};
use calx::backend::{WindowBuilder, Canvas, CanvasBuilder};
use titlescreen::TitleScreen;
use config::Config;

pub static SCREEN_W: u32 = 640;
pub static SCREEN_H: u32 = 360;

pub mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;
mod config;
mod gamescreen;
mod titlescreen;
mod sprite;
mod msg_queue;

pub enum ScreenAction {
    Quit,
    Change(Box<Screen>),
}

pub trait Screen {
    fn update(&mut self, ctx: &mut Canvas) -> Option<ScreenAction>;
}

pub fn version() -> String {
    let next_release = "0.1.0";
    // Set is_release to true for one commit to make a release.
    let is_release = false;

    if is_release {
        format!("{}", next_release)
    } else {
        format!("{}-alpha", next_release)
    }
}

/// Get the application data file path and ensure the path exists.
pub fn app_data_path() -> PathBuf {
    let ret = calx::app_data_path("magog");
    if !ret.exists() {
        // Create the config dir
        // TODO: Handle error
        fs::create_dir_all(&ret).unwrap();
    }
    ret
}

pub fn config() -> Config {
    // TODO: Make this access the actual config through a global value.
    // TODO: Rc<RefCell<Config>> type.
    Default::default()
}

pub fn main() {
    // TODO: Use persistent config object.
    let mut config: config::Config = Default::default();

    // Init the config file
    let cfg_path = config.file_path();
    if cfg_path.exists() {
        config.load(cfg_path).unwrap();
    } else {
        // Write the default config.
        let mut file = File::create(cfg_path).unwrap();
        file.write_all(config.default_file().as_bytes()).unwrap()
    }

    // Read command line arguments, *after* reading the config file. Command
    // line overrules file.
    match config.parse_args(env::args()) {
        Err(e) => {
            println!("{}", e);
            process::exit(1);
        }
        Ok(Some(msg)) => {
            println!("{}", msg);
            return;
        }
        _ => {}
    };

    let window = WindowBuilder::new()
        .set_size(SCREEN_W, SCREEN_H)
        .set_magnify(config.magnify_mode)
        .set_title("Magog")
        .set_fullscreen(config.fullscreen)
        .set_frame_interval(0.030f64)
        .build();

    let mut builder = CanvasBuilder::new();
    tilecache::init(&mut builder);
    let mut state: Box<Screen> = Box::new(TitleScreen::new());
    let mut ctx = builder.build(window);

    loop {
        let result = state.update(&mut ctx);
        ctx.end_frame();
        match result {
            Some(ScreenAction::Quit) => { return; }
            Some(ScreenAction::Change(new_state)) => {
                state = new_state;
            }
            None => {}
        }
    }
}

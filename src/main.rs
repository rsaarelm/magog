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
use calx::backend::{Canvas, CanvasBuilder, Event};
use gamestate::GameState;
use titlestate::TitleState;

pub static SCREEN_W: u32 = 640;
pub static SCREEN_H: u32 = 360;

pub mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;
mod config;
mod gamestate;
mod titlestate;
mod sprite;
mod msg_queue;
mod console;

pub trait State {
    fn process(&mut self, ctx: &mut Canvas, event: Event) -> Option<Transition>;
}

pub enum Transition {
    Game,
    Title,
    Exit,
}

pub fn version() -> String {
    let next_release = "0.1.0";
    let git_hash = include_str!("git_hash.inc");
    // Set is_release to true for one commit to make a release.
    let is_release = false;

    if is_release {
        format!("{}", next_release)
    } else {
        format!("{}-alpha+g{}", next_release, git_hash)
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

pub fn compiler_version() -> String {
    include_str!("../rustc_version.txt").trim().to_string()
}

pub fn screenshot(ctx: &mut Canvas) {
    use time;
    use std::path::{Path};
    use std::fs::{self, File};
    use image;

    let shot = ctx.screenshot();

    let timestamp = time::precise_time_s() as u64;
    // Create screenshot filenames by concatenating the current timestamp in
    // seconds with a running number from 00 to 99. 100 shots per second
    // should be good enough.

    // Default if we fail to generate any of the 100 candidates for this
    // second, just overwrite with the "xx" prefix then.
    let mut filename = format!("magog-{}{}.png", timestamp, "xx");

    // Run through candidates for this second.
    for i in 0..100 {
        let test_filename = format!("magog-{}{:02}.png", timestamp, i);
        if !Path::new(&test_filename).exists() {
            // Thread-safe claiming: create_dir will fail if the dir
            // already exists (it'll exist if another thread is gunning
            // for the same filename and managed to get past us here).
            // At least assuming that create_dir is atomic...
            let squat_dir = format!(".tmp-{}{:02}", timestamp, i);
            if std::fs::create_dir(&squat_dir).is_ok() {
                File::create(&test_filename).unwrap();
                filename = test_filename;
                fs::remove_dir(&squat_dir).unwrap();
                break;
            } else {
                continue;
            }
        }
    }

    let _ = image::save_buffer(&Path::new(&filename), &shot, shot.width(), shot.height(), image::ColorType::RGB(8));
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

    let mut builder = CanvasBuilder::new()
        .set_size(SCREEN_W, SCREEN_H)
        .set_magnify(config.magnify_mode)
        .set_title("Magog")
        .set_fullscreen(config.fullscreen)
        .set_frame_interval(0.030f64);

    tilecache::init(&mut builder);
    let mut state: Box<State> = Box::new(TitleState::new());

    let mut canvas = builder.build();
    loop {
        let event = canvas.next_event();
        match state.process(&mut canvas, event) {
            Some(Transition::Title) => { state = Box::new(TitleState::new()); }
            Some(Transition::Game) => {
                state = Box::new(GameState::new(config)); }
            Some(Transition::Exit) => { break; }
            _ => ()
        }
    }
}

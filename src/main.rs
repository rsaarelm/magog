#![crate_name="magog"]
#![feature(unboxed_closures, plugin)]
#![feature(core, collections, path_ext, old_path, exit_status)]
#![feature(custom_derive)]
#![plugin(rand_macros)]

#[no_link] extern crate rand_macros;
extern crate rand;
extern crate "rustc-serialize" as rustc_serialize;
extern crate num;
extern crate getopts;
extern crate collect;
extern crate image;
extern crate time;
#[macro_use]
extern crate calx;

use std::env;
use std::default::Default;
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
mod world;

pub trait State {
    fn process(&mut self, event: Event) -> Option<Transition>;
}

pub enum Transition {
    Game(Option<u32>),
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

pub fn compiler_version() -> String {
    include_str!("../rustc_version.txt").to_string()
}

pub fn screenshot(ctx: &mut Canvas) {
    use time;
    use std::path;
    use std::fs::{self, File, PathExt};
    use std::thread;
    use std::old_path::Path;
    use image;

    let shot = ctx.screenshot();

    // XXX: If the game is terminated right after taking a screenshot, the
    // screenshotting thread will be stopped and an incomplete image file will
    // result. Not sure if there's a better solution for this than some kind
    // of global worker thread pool where the screenshot thread should be
    // added.
    thread::spawn(move || {
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
            if !path::Path::new(&test_filename).exists() {
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

        let _ = image::save_buffer(&Path::new(filename), &shot, shot.width(), shot.height(), image::ColorType::RGB(8));
    });
}

pub fn main() {
    // TODO: Use persistent config object.
    let mut config: config::Config = Default::default();
    match config.parse_args(env::args()) {
        Err(e) => {
            println!("{}", e);
            env::set_exit_status(1);
            return;
        }
        _ => {}
    };

    let mut canvas = CanvasBuilder::new()
        .set_size(SCREEN_W, SCREEN_H)
        .set_magnify(config.magnify_mode)
        .set_frame_interval(0.030f64);
    tilecache::init(&mut canvas);
    let mut state: Box<State> = Box::new(TitleState::new());

    for evt in canvas.run() {
        match state.process(evt) {
            Some(Transition::Title) => { state = Box::new(TitleState::new()); }
            Some(Transition::Game(seed)) => {
                state = Box::new(GameState::new(seed)); }
            Some(Transition::Exit) => { break; }
            _ => ()
        }
    }
}

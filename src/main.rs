#![crate_name="magog"]
#![feature(path_ext, old_path)]

extern crate image;
extern crate "calx_util" as util;

#[macro_use]
extern crate "calx_backend" as backend;

extern crate world;
extern crate time;

use backend::{Canvas};

use gamestate::GameState;
use titlestate::TitleState;

pub static SCREEN_W: u32 = 640;
pub static SCREEN_H: u32 = 360;

pub mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;
mod gamestate;
mod titlestate;
mod sprite;
mod msg_queue;
mod console;

pub trait State {
    fn process(&mut self, event: backend::Event) -> Option<Transition>;
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
    use std::fs::PathExt;
    use image;

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
            filename = test_filename;
            break;
        }
    }

    let shot = ctx.screenshot();
    let _ = image::save_buffer(&Path::new(filename), shot.as_slice(), shot.width(), shot.height(), image::ColorType::RGB(8));
}

pub fn main() {
    println!("Magog v{}", version());
    println!("{}", compiler_version());
    let mut canvas = backend::CanvasBuilder::new()
        .set_size(SCREEN_W, SCREEN_H)
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

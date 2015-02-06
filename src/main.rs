#![crate_name="magog"]
#![feature(core, io, path, unicode)]

extern crate image;
extern crate "calx_util" as util;
extern crate "calx_backend" as backend;
extern crate world;
extern crate time;

use gamestate::GameState;

pub static SCREEN_W: u32 = 640;
pub static SCREEN_H: u32 = 360;

pub mod drawable;
pub mod tilecache;
pub mod viewutil;
pub mod worldview;
mod gamestate;
//mod titlestate;
mod sprite;
mod msg_queue;

// TODO Fix state machine code.
/*
pub trait State {
    fn process(&mut self, event: event::Event) -> Option<Transition>;
}

pub enum Transition {
    NewState(State),
    Quit,
}
*/

pub fn version() -> String {
    let next_release = "0.1.0";
    let git_hash = include_str!("version.inc");
    // Set is_release to true for one commit to make a release.
    let is_release = false;

    if is_release {
        format!("{}", next_release)
    } else {
        format!("{}-alpha+g{}", next_release, git_hash)
    }
}

pub fn main() {
    let mut canvas = backend::CanvasBuilder::new()
        .set_size(SCREEN_W, SCREEN_H)
        .set_frame_interval(0.030f64);
    tilecache::init(&mut canvas);
    let mut state = GameState::new(None);

    for evt in canvas.run() {
        match state.process(evt) {
            false => { return; }
            _ => ()
        }
    }
}
